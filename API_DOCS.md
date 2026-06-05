# Invoice Service API Documentation

Base URL:

http://localhost:3000

Authentication:

All business-facing endpoints require:

X-API-Key: demo

---

## Authentication

Most endpoints require:

```http
X-API-Key: <business-api-key>
```

Example:

```bash
curl \
-H "X-API-Key: demo" \
http://localhost:3000/customers
```

### Business Creation

Create a business and obtain an API key.

### Request

```http
POST /businesses
```

```json
{
  "name": "Acme Inc"
}
```

### Response

```json
{
  "id": "77e3e263-0e5f-47e7-aba4-2170b14995b3",
  "name": "Acme Inc",
  "api_key": "generated-api-key"
}
```

Use the returned API key in all protected endpoints.

# Customers

## Create Customer

POST /customers

Headers:

X-API-Key: demo
Content-Type: application/json

Request:

```json
{
  "name": "Gopi",
  "email": "gopi@test.com"
}
```

Response (201 Created):

```json
{
  "id": "3dfba3dc-ab11-46ae-90eb-2096958de488",
  "name": "Gopi",
  "email": "gopi@test.com"
}
```

---

## List Customers

GET /customers

Headers:

X-API-Key: demo

Response:

```json
[
  {
    "id": "3dfba3dc-ab11-46ae-90eb-2096958de488",
    "name": "Gopi",
    "email": "gopi@test.com"
  }
]
```

---

# Invoices

## Create Invoice

POST /invoices

Headers:

X-API-Key: demo
Content-Type: application/json

Request:

```json
{
  "customer_id": "3dfba3dc-ab11-46ae-90eb-2096958de488",
  "line_items": [
    {
      "description": "Keyboard",
      "quantity": 1,
      "unit_amount_cents": 3000
    }
  ]
}
```

Response (201 Created):

```json
{
  "id": "a5a7aded-eac4-44b9-a01f-753ba6c338fe",
  "customer_id": "3dfba3dc-ab11-46ae-90eb-2096958de488",
  "total_cents": 3000,
  "state": "open"
}
```

Business Rules:

- Invoice total is calculated from all line items.
- Invoice starts in `open` state.
- Invoice creation is executed inside a database transaction.
- `invoice.created` webhook event is triggered after creation.

---

## List Invoices

GET /invoices

Headers:

X-API-Key: demo

Response:

```json
[
  {
    "id": "a5a7aded-eac4-44b9-a01f-753ba6c338fe",
    "customer_id": "3dfba3dc-ab11-46ae-90eb-2096958de488",
    "total_cents": 3000,
    "state": "open"
  }
]
```

---

## Get Invoice

GET /invoices/{invoice_id}

Headers:

X-API-Key: demo

Response:

```json
{
  "id": "a5a7aded-eac4-44b9-a01f-753ba6c338fe",
  "customer_id": "3dfba3dc-ab11-46ae-90eb-2096958de488",
  "total_cents": 3000,
  "state": "open"
}
```

---

## Pay Invoice

POST /invoices/{invoice_id}/pay

Headers:

X-API-Key: demo
Content-Type: application/json

Request:

```json
{
  "idempotency_key": "payment-001",
  "card_token": "tok_success"
}
```

Response:

```json
{
  "invoice_id": "a5a7aded-eac4-44b9-a01f-753ba6c338fe",
  "status": "success",
  "psp_ref": "psp_123456"
}
```

---

### Supported Test Tokens

#### Success

```json
{
  "card_token": "tok_success"
}
```

Result:

```json
{
  "status": "success"
}
```

---

#### Insufficient Funds

```json
{
  "card_token": "tok_insufficient_funds"
}
```

Result:

```json
{
  "status": "insufficient_funds"
}
```

---

#### Card Declined

```json
{
  "card_token": "tok_card_declined"
}
```

Result:

```json
{
  "status": "card_declined"
}
```

---

#### Timeout

```json
{
  "card_token": "tok_timeout"
}
```

Result:

```json
{
  "status": "timeout"
}
```

---

#### Network Error

```json
{
  "card_token": "tok_network_error"
}
```

Result:

```json
{
  "status": "network_error"
}
```

---

### Payment Guarantees

The payment endpoint implements:

- Idempotency using unique idempotency keys.
- Database transaction protection.
- Row-level locking using `SELECT ... FOR UPDATE`.
- Prevention of double charges.
- PSP integration over HTTP.
- Payment attempt persistence.
- Webhook dispatch on successful payment.

---

# Webhooks

## Register Webhook

POST /webhooks

Headers:

X-API-Key: demo
Content-Type: application/json

Request:

```json
{
  "url": "https://example.com/webhook",
  "secret": "my-secret"
}
```

Response:

```json
{
  "id": "667593dd-04c7-4bf5-9a4a-6784a4280066",
  "url": "https://example.com/webhook"
}
```

---

# Webhook Events

## invoice.created

Triggered when a new invoice is created.

Payload:

```json
{
  "event": "invoice.created",
  "invoice_id": "a5a7aded-eac4-44b9-a01f-753ba6c338fe"
}
```

---

## invoice.paid

Triggered when an invoice is successfully paid.

Payload:

```json
{
  "event": "invoice.paid",
  "invoice_id": "a5a7aded-eac4-44b9-a01f-753ba6c338fe"
}
```

---

# Webhook Deliveries

Webhook delivery attempts are stored in:

```sql
webhook_deliveries
```

Fields:

- id
- webhook_id
- event_type
- payload
- status
- response_code
- created_at

Example:

```sql
SELECT
    event_type,
    status,
    response_code
FROM webhook_deliveries;
```

Result:

```text
invoice.paid | success | 200
invoice.paid | failed  | 500
```

---

# Integration Tests

The project includes automated integration tests.

Run:

```bash
cargo test --test idempotency_test -- --nocapture
```

```bash
cargo test --test concurrency_test -- --nocapture
```

```bash
cargo test --test psp_failure_test -- --nocapture
```

Expected Results:

- Idempotency test passes.
- Concurrency test prevents double payment.
- PSP failure test keeps invoice open.
- All tests return PASS.

---

# Error Responses

400 Bad Request

```json
{
  "error": "invalid_request"
}
```

404 Not Found

```json
{
  "error": "not_found"
}
```

409 Conflict

```json
{
  "error": "already_processed"
}
```

500 Internal Server Error

```json
{
  "error": "internal_server_error"
}
```

# Docker Deployment

Build and start the application:

```bash
docker compose up --build
```

Verify service:

```bash
curl http://localhost:3000/invoices
```

Verify database:

```bash
docker exec -it invoice-service-postgres-1 \
psql postgres://dodo:dodo@localhost:5432/invoice_db
```

Run migrations:

```bash
docker compose exec app sqlx migrate run
```
