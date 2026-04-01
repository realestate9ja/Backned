DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_enum
        WHERE enumlabel = 'suspended'
          AND enumtypid = 'property_status'::regtype
    ) THEN
        ALTER TYPE property_status ADD VALUE 'suspended';
    END IF;
END $$;

ALTER TABLE users
ADD COLUMN IF NOT EXISTS quality_strikes INTEGER NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS fraud_strikes INTEGER NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS listing_restricted_until TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS is_banned BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE reports
ADD COLUMN IF NOT EXISTS violation_type TEXT NOT NULL DEFAULT 'other',
ADD COLUMN IF NOT EXISTS reviewed_by UUID REFERENCES users(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS review_notes TEXT;

ALTER TABLE reports
DROP CONSTRAINT IF EXISTS reports_status_check;

ALTER TABLE reports
ADD CONSTRAINT reports_status_check
CHECK (status IN ('open', 'reviewing', 'upheld', 'dismissed'));

ALTER TABLE reports
DROP CONSTRAINT IF EXISTS reports_violation_type_check;

ALTER TABLE reports
ADD CONSTRAINT reports_violation_type_check
CHECK (violation_type IN ('quality', 'fraud', 'other'));

CREATE INDEX IF NOT EXISTS idx_users_listing_restricted_until ON users(listing_restricted_until);
CREATE INDEX IF NOT EXISTS idx_users_is_banned ON users(is_banned);
CREATE INDEX IF NOT EXISTS idx_reports_violation_type ON reports(violation_type);
CREATE INDEX IF NOT EXISTS idx_reports_reviewed_by ON reports(reviewed_by);
