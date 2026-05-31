DO $$
DECLARE
    constraint_name text;
BEGIN
    SELECT conname
    INTO constraint_name
    FROM pg_constraint
    WHERE conrelid = 'announcements'::regclass
      AND contype = 'c'
      AND pg_get_constraintdef(oid) LIKE '%audience IN%';

    IF constraint_name IS NOT NULL THEN
        EXECUTE format('ALTER TABLE announcements DROP CONSTRAINT %I', constraint_name);
    END IF;
END $$;

ALTER TABLE announcements
    ADD CONSTRAINT announcements_audience_check
    CHECK (audience IN ('all', 'seekers', 'agents', 'landlords', 'admins', 'providers'));
