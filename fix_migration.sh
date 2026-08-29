#!/bin/bash
# Fix migration 19 checksum error on Railway PostgreSQL

# Use the DATABASE_URL from your .env file
psql "postgresql://postgres:JARusIsFblZiWrmxxmZwXPPeEjuaaItV@viaduct.proxy.rlwy.net:34129/railway" <<EOF
-- Check the current migration 19 record
SELECT version, checksum, installed_on FROM _sqlx_migrations WHERE version = 19;

-- Delete the bad record
DELETE FROM _sqlx_migrations WHERE version = 19;

-- Verify it's gone
SELECT version FROM _sqlx_migrations WHERE version = 19;

-- Show all remaining migrations
SELECT version, description, installed_on FROM _sqlx_migrations ORDER BY version;
EOF

echo "Migration fix applied. You can now restart your server."
