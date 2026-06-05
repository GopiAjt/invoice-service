use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCustomerRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct Customer {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct InvoiceItemRequest {
    pub description: String,
    pub quantity: i32,
    pub unit_amount_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceRequest {
    pub customer_id: Uuid,
    pub line_items: Vec<InvoiceItemRequest>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub total_cents: i64,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct InvoiceDetails {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub total_cents: i64,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct PayInvoiceRequest {
    pub idempotency_key: String,
    pub card_token: String,
}

#[derive(Debug, Serialize)]
pub struct PayInvoiceResponse {
    pub invoice_id: Uuid,
    pub status: String,
    pub psp_ref: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PSPChargeRequest {
    pub card_token: String,
    pub amount_cents: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PSPChargeResponse {
    pub success: bool,
    pub psp_ref: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct InvoiceFilter {
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub secret: String,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub id: Uuid,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBusinessRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct BusinessResponse {
    pub id: Uuid,
    pub name: String,
    pub api_key: String,
}