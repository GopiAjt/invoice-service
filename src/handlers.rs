use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use axum::extract::Extension;

use uuid::Uuid;

use crate::{
    models::{
        CreateCustomerRequest, CreateInvoiceRequest, CreateWebhookRequest, Customer,
        InvoiceDetails, InvoiceFilter, InvoiceResponse, PSPChargeRequest, PSPChargeResponse,
        PayInvoiceRequest, PayInvoiceResponse, WebhookResponse,CreateBusinessRequest, BusinessResponse,
    },
    state::AppState,
};

pub async fn create_webhook(
    State(state): State<AppState>,
    Json(payload): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<WebhookResponse>), StatusCode> {
    let webhook_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO webhooks (
            id,
            business_id,
            url,
            secret
        )
        VALUES ($1,$2,$3,$4)
        "#,
    )
    .bind(webhook_id)
    .bind(Uuid::nil()) // Demo Business
    .bind(&payload.url)
    .bind(&payload.secret)
    .execute(&state.db)
    .await
    .map_err(|e| {
        println!("WEBHOOK INSERT ERROR {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        Json(WebhookResponse {
            id: webhook_id,
            url: payload.url,
        }),
    ))
}

pub async fn list_invoices(
    State(state): State<AppState>,
    Query(filter): Query<InvoiceFilter>,
) -> Result<Json<Vec<InvoiceDetails>>, StatusCode> {
    let rows = if let Some(invoice_state) = filter.state {
        sqlx::query_as::<_, (Uuid, Uuid, i64, String)>(
            r#"
            SELECT
                id,
                customer_id,
                total_cents,
                state
            FROM invoices
            WHERE state = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(invoice_state)
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as::<_, (Uuid, Uuid, i64, String)>(
            r#"
            SELECT
                id,
                customer_id,
                total_cents,
                state
            FROM invoices
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let invoices = rows
        .into_iter()
        .map(|row| InvoiceDetails {
            id: row.0,
            customer_id: row.1,
            total_cents: row.2,
            state: row.3,
        })
        .collect();

    Ok(Json(invoices))
}

pub async fn pay_invoice(
    State(state): State<AppState>,
    Path(invoice_id): Path<Uuid>,
    Json(payload): Json<PayInvoiceRequest>,
) -> Result<Json<PayInvoiceResponse>, StatusCode> {
    println!("POST /invoices/{}/pay", invoice_id);

    // --------------------------------------------------
    // IDEMPOTENCY CHECK
    // --------------------------------------------------

    let existing = sqlx::query_as::<_, (String, Option<String>)>(
        r#"
        SELECT status, psp_ref
        FROM payment_attempts
        WHERE idempotency_key = $1
        LIMIT 1
        "#,
    )
    .bind(&payload.idempotency_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some((status, psp_ref)) = existing {
        return Ok(Json(PayInvoiceResponse {
            invoice_id,
            status,
            psp_ref,
        }));
    }

    // --------------------------------------------------
    // START TRANSACTION
    // --------------------------------------------------

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let invoice = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT state,total_cents
        FROM invoices
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(invoice_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    let invoice_state = invoice.0;
    let amount_cents = invoice.1;

    if invoice_state == "paid" {
        return Ok(Json(PayInvoiceResponse {
            invoice_id,
            status: "already_paid".to_string(),
            psp_ref: None,
        }));
    }

    // --------------------------------------------------
    // CALL PSP OVER HTTP
    // --------------------------------------------------

    let client = reqwest::Client::new();

    let psp_response: PSPChargeResponse = client
        .post("http://localhost:3000/psp/charge")
        .json(&PSPChargeRequest {
            card_token: payload.card_token.clone(),
            amount_cents,
        })
        .send()
        .await
        .map_err(|e| {
            println!("PSP HTTP ERROR {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .json::<PSPChargeResponse>()
        .await
        .map_err(|e| {
            println!("PSP JSON ERROR {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let success = psp_response.success;
    let status = psp_response.status.clone();

    let psp_ref = if psp_response.psp_ref.is_empty() {
        None
    } else {
        Some(psp_response.psp_ref.clone())
    };

    // --------------------------------------------------
    // RECORD PAYMENT ATTEMPT
    // --------------------------------------------------

    sqlx::query(
        r#"
        INSERT INTO payment_attempts (
            id,
            invoice_id,
            status,
            idempotency_key,
            psp_ref
        )
        VALUES ($1,$2,$3,$4,$5)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(invoice_id)
    .bind(&status)
    .bind(&payload.idempotency_key)
    .bind(psp_ref.clone())
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // --------------------------------------------------
    // MARK INVOICE PAID
    // --------------------------------------------------

    if success {
        sqlx::query(
            r#"
            UPDATE invoices
            SET state='paid'
            WHERE id=$1
            "#,
        )
        .bind(invoice_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // --------------------------------------------------
    // SEND WEBHOOKS + LOG DELIVERIES
    // --------------------------------------------------

    let event_type = if success {
        "invoice.paid"
    } else {
        "invoice.payment_failed"
    };

    let webhooks = sqlx::query_as::<_, (Uuid, String, String)>(
        r#"
        SELECT id, url, secret
        FROM webhooks
        "#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    for (webhook_id, url, _secret) in webhooks {
        let payload_json = serde_json::json!({
            "event": event_type,
            "invoice_id": invoice_id,
        });

        println!("Sending webhook event={} to {}", event_type, url);

        let response = client.post(&url).json(&payload_json).send().await;

        let (delivery_status, response_code) = match response {
            Ok(resp) => {
                let code = resp.status().as_u16() as i32;

                if resp.status().is_success() {
                    ("success", Some(code))
                } else {
                    ("failed", Some(code))
                }
            }

            Err(_) => ("failed", None),
        };

        let _ = sqlx::query(
            r#"
            INSERT INTO webhook_deliveries (
                id,
                webhook_id,
                event_type,
                payload,
                status,
                response_code
            )
            VALUES ($1,$2,$3,$4,$5,$6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(webhook_id)
        .bind(event_type)
        .bind(payload_json)
        .bind(delivery_status)
        .bind(response_code)
        .execute(&state.db)
        .await;
    }

    Ok(Json(PayInvoiceResponse {
        invoice_id,
        status,
        psp_ref,
    }))
}

pub async fn get_invoice(
    State(state): State<AppState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<InvoiceDetails>, StatusCode> {
    let row = sqlx::query_as::<_, (Uuid, Uuid, i64, String)>(
        r#"
        SELECT
            id,
            customer_id,
            total_cents,
            state
        FROM invoices
        WHERE id = $1
        "#,
    )
    .bind(invoice_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        println!("GET INVOICE ERROR: {:?}", e);
        StatusCode::NOT_FOUND
    })?;

    Ok(Json(InvoiceDetails {
        id: row.0,
        customer_id: row.1,
        total_cents: row.2,
        state: row.3,
    }))
}

pub async fn create_invoice(
    State(state): State<AppState>,
    Json(payload): Json<CreateInvoiceRequest>,
) -> Result<(StatusCode, Json<InvoiceResponse>), StatusCode> {
    println!("POST /invoices called");

    // --------------------------------------------------
    // VALIDATION
    // --------------------------------------------------

    if payload.line_items.is_empty() {
        println!("Invoice must contain at least one item");
        return Err(StatusCode::BAD_REQUEST);
    }

    for item in &payload.line_items {
        if item.quantity <= 0 {
            println!("Quantity must be greater than zero");
            return Err(StatusCode::BAD_REQUEST);
        }

        if item.unit_amount_cents <= 0 {
            println!("Amount must be greater than zero");
            return Err(StatusCode::BAD_REQUEST);
        }

        if item.description.trim().is_empty() {
            println!("Description cannot be empty");
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let invoice_id = Uuid::new_v4();

    let total_cents: i64 = payload
        .line_items
        .iter()
        .map(|item| item.quantity as i64 * item.unit_amount_cents)
        .sum();

    // --------------------------------------------------
    // START TRANSACTION
    // --------------------------------------------------

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // --------------------------------------------------
    // CREATE INVOICE
    // --------------------------------------------------

    sqlx::query(
        r#"
        INSERT INTO invoices (
            id,
            customer_id,
            total_cents,
            state
        )
        VALUES ($1,$2,$3,$4)
        "#,
    )
    .bind(invoice_id)
    .bind(payload.customer_id)
    .bind(total_cents)
    .bind("open")
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        println!("INVOICE ERROR {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // --------------------------------------------------
    // CREATE INVOICE ITEMS
    // --------------------------------------------------

    for item in &payload.line_items {
        sqlx::query(
            r#"
            INSERT INTO invoice_items (
                id,
                invoice_id,
                description,
                quantity,
                unit_amount_cents
            )
            VALUES ($1,$2,$3,$4,$5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(invoice_id)
        .bind(&item.description)
        .bind(item.quantity)
        .bind(item.unit_amount_cents)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            println!("ITEM INSERT ERROR {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    // --------------------------------------------------
    // COMMIT
    // --------------------------------------------------

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("Invoice created");

    trigger_webhook_event(&state, "invoice.created", invoice_id).await;
    Ok((
        StatusCode::CREATED,
        Json(InvoiceResponse {
            id: invoice_id,
            customer_id: payload.customer_id,
            total_cents,
            state: "open".to_string(),
        }),
    ))
}

pub async fn create_customer(
    State(state): State<AppState>,
    Extension(business_id): Extension<Uuid>,
    Json(payload): Json<CreateCustomerRequest>,
) -> Result<(StatusCode, Json<Customer>), StatusCode>  {
    
    println!("POST /customers called");
    println!("Payload: {:?}", payload);

    if payload.email.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if !payload.email.contains('@') {
        return Err(StatusCode::BAD_REQUEST);
    }

    let customer_id = Uuid::new_v4();

    let result = sqlx::query(
        r#"
        INSERT INTO customers (
            id,
            business_id,
            name,
            email
        )
        VALUES ($1,$2,$3,$4)
        "#,
    )
    .bind(customer_id)
    .bind(business_id)
    .bind(&payload.name)
    .bind(&payload.email)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            println!("Customer inserted successfully");
        }
        Err(e) => {
            println!("=========================");
            println!("DATABASE ERROR:");
            println!("{:?}", e);
            println!("=========================");

            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(Customer {
            id: customer_id,
            name: payload.name,
            email: payload.email,
        }),
    ))
}

pub async fn get_customer(
    State(state): State<AppState>,
    Path(customer_id): Path<Uuid>,
) -> Result<Json<Customer>, StatusCode> {
    let row = sqlx::query_as::<_, (Uuid, String, String)>(
        r#"
        SELECT id, name, email
        FROM customers
        WHERE id = $1
        "#,
    )
    .bind(customer_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(Customer {
        id: row.0,
        name: row.1,
        email: row.2,
    }))
}

pub async fn list_customers(
    State(state): State<AppState>,
) -> Result<Json<Vec<Customer>>, StatusCode> {
    let rows = sqlx::query_as::<_, (Uuid, String, String)>(
        r#"
        SELECT id,name,email
        FROM customers
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        println!("LIST CUSTOMERS ERROR: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let customers = rows
        .into_iter()
        .map(|(id, name, email)| Customer { id, name, email })
        .collect();

    Ok(Json(customers))
}

async fn trigger_webhook_event(state: &AppState, event_type: &str, invoice_id: Uuid) {
    let webhooks = sqlx::query_as::<_, (Uuid, String)>(
        r#"
        SELECT id, url
        FROM webhooks
        "#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (webhook_id, url) in webhooks {
        let payload_json = serde_json::json!({
            "event": event_type,
            "invoice_id": invoice_id
        });

        let response = reqwest::Client::new()
            .post(&url)
            .json(&payload_json)
            .send()
            .await;

        let (delivery_status, response_code) = match response {
            Ok(resp) => ("success", Some(resp.status().as_u16() as i32)),

            Err(_) => ("failed", None),
        };

        let _ = sqlx::query(
            r#"
            INSERT INTO webhook_deliveries (
                id,
                webhook_id,
                event_type,
                payload,
                status,
                response_code
            )
            VALUES ($1,$2,$3,$4,$5,$6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(webhook_id)
        .bind(event_type)
        .bind(payload_json)
        .bind(delivery_status)
        .bind(response_code)
        .execute(&state.db)
        .await;
    }
}

pub async fn create_business(
    State(state): State<AppState>,
    Json(payload): Json<CreateBusinessRequest>,
) -> Result<(StatusCode, Json<BusinessResponse>), StatusCode> {

    let business_id = Uuid::new_v4();

    let api_key = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO businesses (
            id,
            name,
            api_key_hash
        )
        VALUES ($1,$2,$3)
        "#
    )
    .bind(business_id)
    .bind(&payload.name)
    .bind(&api_key)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(BusinessResponse {
            id: business_id,
            name: payload.name,
            api_key,
        }),
    ))
}