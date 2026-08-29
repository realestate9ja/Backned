-- Add indexes to optimize property listing query performance
-- Index on properties table for published/rented/sold status and creation date
CREATE INDEX IF NOT EXISTS idx_properties_status_created_at 
ON properties(status, created_at DESC) 
WHERE status IN ('published', 'rented_out', 'sold_out', 'in_use');

-- Index on properties location for text search
CREATE INDEX IF NOT EXISTS idx_properties_location 
ON properties(location);

-- Index on properties price for range filtering
CREATE INDEX IF NOT EXISTS idx_properties_price 
ON properties(price)
WHERE status IN ('published', 'rented_out', 'sold_out', 'in_use');

-- Index on properties status for faster filtering
CREATE INDEX IF NOT EXISTS idx_properties_status 
ON properties(status)
WHERE status IN ('published', 'rented_out', 'sold_out', 'in_use');

-- Index on property_views for faster stats calculation
CREATE INDEX IF NOT EXISTS idx_property_views_property_id 
ON property_views(property_id);

-- Index on offers for faster stats calculation
CREATE INDEX IF NOT EXISTS idx_offers_property_id 
ON offers(property_id);

-- Create materialized view for property stats (run refresh periodically)
DROP MATERIALIZED VIEW IF EXISTS property_stats_cache CASCADE;

CREATE MATERIALIZED VIEW property_stats_cache AS
SELECT 
    p.id as property_id,
    COALESCE(pv.view_count, 0) as view_count,
    COALESCE(po.offer_count, 0) as offer_count
FROM properties p
LEFT JOIN (
    SELECT property_id, COUNT(*)::bigint as view_count
    FROM property_views
    GROUP BY property_id
) pv ON p.id = pv.property_id
LEFT JOIN (
    SELECT property_id, COUNT(*)::bigint as offer_count
    FROM offers
    GROUP BY property_id
) po ON p.id = po.property_id;

CREATE UNIQUE INDEX idx_property_stats_cache_id ON property_stats_cache(property_id);

