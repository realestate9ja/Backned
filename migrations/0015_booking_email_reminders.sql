ALTER TABLE bookings
    ADD COLUMN IF NOT EXISTS seeker_upcoming_email_sent_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS provider_upcoming_email_sent_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS seeker_past_due_email_sent_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS provider_past_due_email_sent_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_bookings_scheduled_for_status
    ON bookings (scheduled_for, status);
