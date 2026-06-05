-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE businesses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    api_key_hash TEXT NOT NULL
);

CREATE TABLE customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id),

    name TEXT NOT NULL,
    email TEXT NOT NULL
);

CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    customer_id UUID NOT NULL REFERENCES customers(id),

    total_cents BIGINT NOT NULL,

    state TEXT NOT NULL,

    due_date TIMESTAMP,

    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE invoice_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    invoice_id UUID NOT NULL REFERENCES invoices(id),

    description TEXT NOT NULL,

    quantity INTEGER NOT NULL,

    unit_amount_cents BIGINT NOT NULL
);

CREATE TABLE payment_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    invoice_id UUID NOT NULL REFERENCES invoices(id),

    status TEXT NOT NULL,

    idempotency_key TEXT UNIQUE,

    psp_ref TEXT,

    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    business_id UUID NOT NULL REFERENCES businesses(id),

    url TEXT NOT NULL,

    secret TEXT NOT NULL
);