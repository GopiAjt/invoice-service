use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn psp_failure_keeps_invoice_open() {
    let client = Client::new();

    let invoice = client
        .post("http://localhost:3000/invoices")
        .header("X-API-Key", "58a3f22a-db61-4992-ab03-825e9952e961")
        .json(&json!({
            "customer_id":"1e631626-06c9-4e9d-8539-224a757a3e64",
            "line_items":[
                {
                    "description":"PSP Failure Test",
                    "quantity":1,
                    "unit_amount_cents":2000
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

    let payment = client
        .post(format!("http://localhost:3000/invoices/{}/pay", invoice_id))
        .header("X-API-Key", "58a3f22a-db61-4992-ab03-825e9952e961")
        .json(&json!({
            "idempotency_key":"psp-failure-001",
            "card_token":"tok_card_declined"
        }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_eq!(payment["status"], "card_declined");

    let invoice_after = client
        .get(format!("http://localhost:3000/invoices/{}", invoice_id))
        .header("X-API-Key", "58a3f22a-db61-4992-ab03-825e9952e961")
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_eq!(invoice_after["state"], "open");

    println!("PSP failure test passed");
}
