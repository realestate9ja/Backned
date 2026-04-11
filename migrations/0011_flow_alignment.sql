DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'user_role' AND e.enumlabel = 'unassigned'
    ) THEN
        ALTER TYPE user_role ADD VALUE 'unassigned';
    END IF;
END $$;

ALTER TABLE users
    ALTER COLUMN role SET DEFAULT 'unassigned';

CREATE TABLE IF NOT EXISTS email_verification_codes (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    purpose TEXT NOT NULL DEFAULT 'verify_email',
    code TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_verification_codes_user_id ON email_verification_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_email_verification_codes_email ON email_verification_codes(email);
CREATE INDEX IF NOT EXISTS idx_email_verification_codes_expires_at ON email_verification_codes(expires_at);

ALTER TABLE properties
    ADD COLUMN IF NOT EXISTS views BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS inquiries BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS sqft BIGINT,
    ADD COLUMN IF NOT EXISTS state TEXT;

ALTER TABLE maintenance_requests
    ADD COLUMN IF NOT EXISTS reported_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;
