DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_enum
        WHERE enumlabel = 'hidden'
          AND enumtypid = 'property_status'::regtype
    ) THEN
        ALTER TYPE property_status ADD VALUE 'hidden';
    END IF;
END $$;
