ALTER TABLE posts
ADD COLUMN IF NOT EXISTS request_title TEXT NOT NULL DEFAULT '',
ADD COLUMN IF NOT EXISTS area TEXT NOT NULL DEFAULT '',
ADD COLUMN IF NOT EXISTS property_type TEXT NOT NULL DEFAULT '',
ADD COLUMN IF NOT EXISTS bedrooms INTEGER NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS min_budget BIGINT NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS max_budget BIGINT NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS pricing_preference TEXT NOT NULL DEFAULT '',
ADD COLUMN IF NOT EXISTS desired_features TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'active';

UPDATE posts
SET request_title = CASE WHEN request_title = '' THEN 'Housing request' ELSE request_title END;

UPDATE posts
SET area = CASE WHEN area = '' THEN location ELSE area END;

UPDATE posts
SET property_type = CASE WHEN property_type = '' THEN 'apartment' ELSE property_type END;

UPDATE posts
SET min_budget = CASE WHEN min_budget = 0 THEN budget ELSE min_budget END;

UPDATE posts
SET max_budget = CASE WHEN max_budget = 0 THEN budget ELSE max_budget END;

UPDATE posts
SET pricing_preference = CASE WHEN pricing_preference = '' THEN 'monthly' ELSE pricing_preference END;

CREATE TABLE IF NOT EXISTS response_properties (
    response_id UUID NOT NULL REFERENCES responses(id) ON DELETE CASCADE,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    PRIMARY KEY (response_id, property_id)
);

CREATE INDEX IF NOT EXISTS idx_posts_request_title ON posts(request_title);
CREATE INDEX IF NOT EXISTS idx_posts_property_type ON posts(property_type);
CREATE INDEX IF NOT EXISTS idx_posts_status ON posts(status);
CREATE INDEX IF NOT EXISTS idx_response_properties_property_id ON response_properties(property_id);
