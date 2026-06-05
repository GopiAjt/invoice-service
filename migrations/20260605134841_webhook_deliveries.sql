-- Add migration script here
CREATE TABLE webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    webhook_id UUID NOT NULL REFERENCES webhooks(id),

    event_type TEXT NOT NULL,

    payload JSONB NOT NULL,

    status TEXT NOT NULL,

    response_code INTEGER,

    created_at TIMESTAMP DEFAULT NOW()
);