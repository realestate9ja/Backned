-- Table to track when properties are temporarily rented and when they'll be available again
CREATE TABLE IF NOT EXISTS property_rental_periods (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    rented_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    available_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_property_rental_periods_property_id 
ON property_rental_periods(property_id);

CREATE INDEX IF NOT EXISTS idx_property_rental_periods_available_at 
ON property_rental_periods(available_at) 
WHERE available_at IS NOT NULL;
