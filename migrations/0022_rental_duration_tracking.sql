-- Add duration tracking for rental and shortlet properties
-- This tracks how long a property will be occupied after outcome confirmation

ALTER TABLE properties
ADD COLUMN IF NOT EXISTS rental_duration_months INTEGER,
ADD COLUMN IF NOT EXISTS rental_start_date TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS rental_end_date TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS shortlet_duration_days INTEGER,
ADD COLUMN IF NOT EXISTS shortlet_start_date TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS shortlet_end_date TIMESTAMPTZ;

-- Update bookings to track the duration chosen by seeker/provider
ALTER TABLE bookings
ADD COLUMN IF NOT EXISTS agreed_duration_value INTEGER,
ADD COLUMN IF NOT EXISTS agreed_duration_unit TEXT CHECK (agreed_duration_unit IN ('months', 'days'));

-- Add index for queries on rental/shortlet end dates to check availability
CREATE INDEX IF NOT EXISTS idx_properties_rental_end_date ON properties(rental_end_date);
CREATE INDEX IF NOT EXISTS idx_properties_shortlet_end_date ON properties(shortlet_end_date);
