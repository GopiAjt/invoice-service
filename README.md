# Invoice Service

A production-style invoice management service built in Rust using Axum, SQLx, PostgreSQL, and Docker.

This project was implemented as part of a backend engineering assessment and demonstrates:

- Customer management
- Invoice creation
- Payment processing
- PSP integration
- Idempotent payments
- Concurrency-safe payment handling
- API key authentication
- Webhook registration
- Webhook delivery logging
- Automated integration testing

---

# Tech Stack

- Rust
- Axum
- SQLx
- PostgreSQL
- Docker
- Reqwest
- Tokio
- UUID

---

# Features

## Customers

- Create customer
- Get customer by ID
- List customers

## Invoices

- Create invoice
- Get invoice by ID
- List invoices
- Filter invoices by state

## Payments

- Pay invoice
- Idempotency support
- PSP integration via HTTP
- Payment attempt tracking
- Concurrency-safe payment processing

## Authentication

- API Key based authentication
- Business scoped access

## Webhooks

- Register webhook endpoints
- Send invoice.paid events
- Store webhook delivery history

---

# Architecture

Client
↓
Axum API
↓
PostgreSQL

Payment Flow:

Invoice
↓
Pay Endpoint
↓
Mock PSP
↓
Update Invoice
↓
Store Payment Attempt
↓
Send Webhook
↓
Store Delivery Record

---

# Database Schema

Tables:

- businesses
- customers
- invoices
- invoice_items
- payment_attempts
- webhooks
- webhook_deliveries

---

# Running Locally

## 1. Clone Repository

```bash
git clone <repository-url>
cd invoice-service
```

## 2. Start PostgreSQL

```bash
docker compose up -d postgres
```

## 3. Run Migrations

```bash
sqlx migrate run
```

## 4. Start Service

```bash
cargo run
```

Server starts on:

```text
http://localhost:3000
```

---

## Running with Docker

### Build and Start

```bash
docker compose up --build
```

Application:

```text
http://localhost:3000
```

PostgreSQL:

```text
postgres://dodo:dodo@localhost:5432/invoice_db
```

### Run Migrations

```bash
docker compose exec app sqlx migrate run
```

### Stop

```bash
docker compose down
```

### Remove Database Volume

```bash
docker compose down -v
```

## Running Tests

Start the application:

```bash
docker compose up
```

Open another terminal:

```bash
cargo test --test idempotency_test

cargo test --test concurrency_test

cargo test --test psp_failure_test
```

Expected Result:

```text
3 tests passed
```

# Demo API Key

```text
X-API-Key: demo
```

---

# Example Requests

## Create Business

```bash
curl -X POST http://localhost:3000/businesses \
-H "Content-Type: application/json" \
-d '{
  "name": "Acme Inc"
}'
```

"api_key":"8f3ee58b-099e-4b66-9cd3-3075b145e630"

## Create Customer

```bash
curl -X POST http://localhost:3000/customers \
-H "X-API-Key: 8f3ee58b-099e-4b66-9cd3-3075b145e630" \
-H "Content-Type: application/json" \
-d '{
  "name":"John Doe",
  "email":"john@example.com"
}'
```

"id":"1e631626-06c9-4e9d-8539-224a757a3e64"

## Create Invoice

```bash
curl -H "X-API-Key: 8f3ee58b-099e-4b66-9cd3-3075b145e630" \
-X POST http://localhost:3000/invoices \
-H "Content-Type: application/json" \
-d '{
  "customer_id":"1a1d4c96-a1a3-4d35-8743-751146118c30",
  "line_items":[
    {
      "description":"Laptop",
      "quantity":1,
      "unit_amount_cents":10000
    }
  ]
}'
```

## Create a webhook

```bash
curl -X POST http://localhost:3000/webhooks \
-H "X-API-Key: 8f3ee58b-099e-4b66-9cd3-3075b145e630" \
-H "Content-Type: application/json" \
-d '{
  "url":"https://webhook.site/your-id",
  "secret":"test-secret"
}'
```

---

## Pay Invoice

```bash
curl -H "X-API-Key: 8f3ee58b-099e-4b66-9cd3-3075b145e630" \
-X POST http://localhost:3000/invoices/58a086f1-5b43-48f9-bdbe-83cdb05e781c/pay \
-H "Content-Type: application/json" \
-d '{
  "idempotency_key":"payment-001",
  "card_token":"tok_success"
}'
```

```bash
curl -H "X-API-Key: 8f3ee58b-099e-4b66-9cd3-3075b145e630" \
-X POST http://localhost:3000/invoices/58a086f1-5b43-48f9-bdbe-83cdb05e781c/pay \
-H "Content-Type: application/json" \
-d '{
  "idempotency_key": "payment-002",
  "card_token": "tok_card_declined"
}'
```

---

# Mock PSP Tokens

Successful payment:

```text
tok_success
```

Failures:

```text
tok_card_declined
tok_insufficient_funds
tok_timeout
tok_network_error
```

---

# Automated Tests

Run all tests:

```bash
cargo test -- --nocapture
```

Run individual tests:

```bash
cargo test --test idempotency_test -- --nocapture
cargo test --test concurrency_test -- --nocapture
cargo test --test psp_failure_test -- --nocapture
```

---

# Test Coverage

## Idempotency Test

Verifies duplicate payment requests return the original result without creating additional payment attempts.

## Concurrency Test

Verifies simultaneous payment requests cannot double-charge an invoice.

## PSP Failure Test

Verifies failed PSP responses do not mark invoices as paid.

---

# Assumptions

- Single PostgreSQL instance
- Mock PSP used for assessment purposes
- API key authentication used instead of OAuth/JWT
- Webhooks are delivered best-effort
- Webhook retries are not implemented

---

# Future Improvements

- Webhook retry mechanism
- Dead-letter queue
- Background job processing
- Structured logging
- OpenTelemetry tracing
- Rate limiting
- Metrics endpoint
- Pagination support
- API versioning

---

# Video Demo

- https://drive.google.com/file/d/1YT9anTWsNoSOLXEIBlGWhU4c1raOBi6S/view?usp=sharing

# Author

Gopi Ajatrao
