ALTER TABLE users
DROP CONSTRAINT IF EXISTS users_verification_status_check;

ALTER TABLE users
ADD CONSTRAINT users_verification_status_check
CHECK (verification_status IN ('not_required', 'pending', 'submitted', 'in_review', 'approved', 'verified', 'rejected'));

UPDATE users
SET verification_status = CASE
    WHEN verification_status = 'verified' THEN 'approved'
    ELSE verification_status
END;
