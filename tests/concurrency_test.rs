use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn concurrent_payment_requests() {
    let client = Client::new();

    let invoice = client
        .post("http://localhost:3000/invoices")
        .header("X-API-Key", "demo")
        .json(&json!({
            "customer_id":"3dfba3dc-ab11-46ae-90eb-2096958de488",
            "line_items":[
                {
                    "description":"Concurrency Test",
                    "quantity":1,
                    "unit_amount_cents":3000
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

    let url = format!("http://localhost:3000/invoices/{}/pay", invoice_id);

    let payload1 = json!({
        "idempotency_key":"concurrent-1",
        "card_token":"tok_success"
    });

    let payload2 = json!({
        "idempotency_key":"concurrent-2",
        "card_token":"tok_success"
    });

    let req1 = client
        .post(&url)
        .header("X-API-Key", "demo")
        .json(&payload1)
        .send();

    let req2 = client
        .post(&url)
        .header("X-API-Key", "demo")
        .json(&payload2)
        .send();

    let (r1, r2) = tokio::join!(req1, req2);

    let p1: serde_json::Value = r1.unwrap().json().await.unwrap();
    let p2: serde_json::Value = r2.unwrap().json().await.unwrap();

    let s1 = p1["status"].as_str().unwrap();
    let s2 = p2["status"].as_str().unwrap();

    println!("Response1: {}", s1);
    println!("Response2: {}", s2);

    assert!((s1 == "success" && s2 != "success") || (s2 == "success" && s1 != "success"));

    println!("Concurrency test passed");
}
