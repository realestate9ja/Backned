DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_enum
        WHERE enumlabel = 'admin'
          AND enumtypid = 'user_role'::regtype
    ) THEN
        ALTER TYPE user_role ADD VALUE 'admin';
    END IF;
END $$;

ALTER TABLE users
ADD COLUMN IF NOT EXISTS verification_status TEXT NOT NULL DEFAULT 'not_required',
ADD COLUMN IF NOT EXISTS verification_notes TEXT,
ADD COLUMN IF NOT EXISTS verified_at TIMESTAMPTZ;

ALTER TABLE users
DROP CONSTRAINT IF EXISTS users_verification_status_check;

ALTER TABLE users
ADD CONSTRAINT users_verification_status_check
CHECK (verification_status IN ('not_required', 'pending', 'verified', 'rejected'));

UPDATE users
SET verification_status = CASE
    WHEN role = 'agent' THEN 'pending'
    ELSE 'not_required'
END
WHERE verification_status = 'not_required';

CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_verification_status ON users(verification_status);
