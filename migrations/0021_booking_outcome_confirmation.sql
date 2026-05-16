ALTER TYPE property_status ADD VALUE IF NOT EXISTS 'rented_out';
ALTER TYPE property_status ADD VALUE IF NOT EXISTS 'sold_out';
ALTER TYPE property_status ADD VALUE IF NOT EXISTS 'in_use';

ALTER TABLE bookings
ADD COLUMN IF NOT EXISTS seeker_outcome TEXT,
ADD COLUMN IF NOT EXISTS seeker_outcome_note TEXT,
ADD COLUMN IF NOT EXISTS seeker_outcome_confirmed_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS provider_outcome TEXT,
ADD COLUMN IF NOT EXISTS provider_outcome_note TEXT,
ADD COLUMN IF NOT EXISTS provider_outcome_confirmed_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS outcome_resolution TEXT,
ADD COLUMN IF NOT EXISTS outcome_follow_up_required BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS listing_outcome_applied BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX IF NOT EXISTS idx_bookings_outcome_follow_up_required ON bookings(outcome_follow_up_required);
