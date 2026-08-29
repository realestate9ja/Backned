-- Add confirmation timestamp to bookings table
ALTER TABLE bookings
ADD COLUMN confirmed_at TIMESTAMPTZ;

-- Index for faster queries on confirmed bookings
CREATE INDEX IF NOT EXISTS idx_bookings_confirmed_at ON bookings(confirmed_at);
