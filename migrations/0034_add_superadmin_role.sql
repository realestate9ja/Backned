-- Add SuperAdmin role to user_role enum
-- SuperAdmin can assign roles to users (higher privilege than Admin)
ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'super_admin';
