use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn payment_is_idempotent() {
    let client = Client::new();

    // Create invoice
    let invoice = client
        .post("http://localhost:3000/invoices")
        .header("X-API-Key", "demo")
        .json(&json!({
            "customer_id":"3dfba3dc-ab11-46ae-90eb-2096958de488",
            "line_items":[
                {
                    "description":"Idempotency Test",
                    "quantity":1,
                    "unit_amount_cents":1000
                }
            ]
        }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    let invoice_id = invoice["id"].as_str().unwrap();

    let payload = json!({
        "idempotency_key":"test-idempotent-001",
        "card_token":"tok_success"
    });

    let first = client
        .post(format!("http://localhost:3000/invoices/{}/pay", invoice_id))
        .header("X-API-Key", "demo")
        .json(&payload)
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    let second = client
        .post(format!("http://localhost:3000/invoices/{}/pay", invoice_id))
        .header("X-API-Key", "demo")
        .json(&payload)
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_eq!(first["status"], "success");
    assert_eq!(second["status"], "success");

    println!("Idempotency test passed");
}
