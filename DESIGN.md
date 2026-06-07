# Invoice Service — System Design

## Summary

This is a Rust-based invoice payment system built with:

- Axum (HTTP layer)
- PostgreSQL (system of record)
- SQLx (query layer)
- Tokio (async runtime)

It supports:

- Multi-tenant billing (businesses)
- Invoice creation and lifecycle management
- Idempotent payments via PSP integration
- Strong concurrency safety guarantees
- Webhook delivery for downstream eventing

Core principle:

> PostgreSQL is the source of truth for correctness. Everything else is derived.

---

# 1. Data Model

## 1.1 Design Philosophy

The schema is optimized for correctness under concurrency, simple transactional boundaries, and predictable query paths.

---

## 1.2 Tables

### businesses

- id (UUID, PK)
- name (text)
- api_key_hash (text, UNIQUE)

### customers

- id (UUID, PK)
- business_id (UUID, indexed)
- name (text)
- email (text)

### invoices (aggregate root)

- id (UUID, PK)
- customer_id (UUID, indexed)
- total_cents (int)
- state (text)
- due_date (timestamp)
- created_at (timestamp)

### invoice_items

- id (UUID, PK)
- invoice_id (UUID, indexed)
- description (text)
- quantity (int)
- unit_amount_cents (int)

### payment_attempts

- id (UUID, PK)
- invoice_id (UUID, indexed)
- status (text)
- idempotency_key (text, UNIQUE)
- psp_ref (text)
- created_at (timestamp)

### webhooks

- id (UUID, PK)
- business_id (UUID, indexed)
- url (text)
- secret (text)

### webhook_deliveries

- id (UUID, PK)
- webhook_id (UUID, indexed)
- event_type (text)
- payload (jsonb)
- status (text)
- response_code (int)
- created_at (timestamp)

---

# 2. Invoice State Machine

## States

        +--------+
        | OPEN   |
        +--------+
            |
            | payment_success
            v
        +--------+
        | PAID   |  (terminal)
        +--------+

            |
            | payment_failed (PSP decline)
            v
        +------------+
        | OPEN       | (remains open)
        +------------+

Rules:

- PAID is terminal
- OPEN is retryable

---

# 3. Payment Correctness

Handled via:

- PostgreSQL row locks
- idempotency keys

Key guarantees:

- No double charges
- Safe concurrent requests
- PSP failures safely handled

---

# 4. Webhooks

- HMAC-SHA256 signing
- retry policy with backoff
- async delivery (non-blocking payment flow)

---

# 5. API Keys

- SHA-256 hashed storage
- X-API-Key authentication
- rotation supported

---

# 6. What was NOT built

- proper state machine for invoice lifecycle
- Refund system
- Kafka/event streaming
- Partial payments
- Multi-PSP routing

---

# 7. Production Gaps

- Observability missing
- Background workers needed
- Rate limiting missing
- No monitoring/alerting
- No CI/CD pipelines
