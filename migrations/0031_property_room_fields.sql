ALTER TABLE properties
    ADD COLUMN IF NOT EXISTS bedrooms INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS bathrooms INTEGER NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS bedrooms_label TEXT,
    ADD COLUMN IF NOT EXISTS property_category TEXT;

UPDATE properties
SET bedrooms = CASE
        WHEN bedrooms < 0 THEN 0
        ELSE bedrooms
    END,
    bathrooms = CASE
        WHEN bathrooms < 1 THEN 1
        ELSE bathrooms
    END;
