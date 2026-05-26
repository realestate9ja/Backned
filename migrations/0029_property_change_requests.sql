CREATE TABLE IF NOT EXISTS property_change_requests (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    requested_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    request_type TEXT NOT NULL CHECK (request_type IN ('price_increase')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    current_price BIGINT NOT NULL,
    requested_price BIGINT NOT NULL,
    reason TEXT,
    review_note TEXT,
    old_value_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    new_value_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at TIMESTAMPTZ,
    reviewed_by UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_property_change_requests_property_id
    ON property_change_requests(property_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_property_change_requests_status
    ON property_change_requests(status, created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS idx_property_change_requests_single_pending_price
    ON property_change_requests(property_id, request_type)
    WHERE status = 'pending';
