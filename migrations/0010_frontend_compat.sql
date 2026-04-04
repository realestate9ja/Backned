DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'user_role' AND e.enumlabel = 'buyer'
    ) AND NOT EXISTS (
        SELECT 1
        FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'user_role' AND e.enumlabel = 'seeker'
    ) THEN
        ALTER TYPE user_role RENAME VALUE 'buyer' TO 'seeker';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'property_status' AND e.enumlabel = 'pending_review'
    ) THEN
        ALTER TYPE property_status ADD VALUE 'pending_review';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'property_status' AND e.enumlabel = 'active'
    ) THEN
        ALTER TYPE property_status ADD VALUE 'active';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'property_status' AND e.enumlabel = 'paused'
    ) THEN
        ALTER TYPE property_status ADD VALUE 'paused';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'property_status' AND e.enumlabel = 'archived'
    ) THEN
        ALTER TYPE property_status ADD VALUE 'archived';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'property_status' AND e.enumlabel = 'rejected'
    ) THEN
        ALTER TYPE property_status ADD VALUE 'rejected';
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS refresh_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);

CREATE TABLE IF NOT EXISTS profiles (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    full_name TEXT NOT NULL,
    phone TEXT,
    city TEXT,
    avatar_url TEXT,
    bio TEXT,
    onboarding_completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS seeker_profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    preferred_city TEXT,
    preferred_accommodation_type TEXT,
    preferred_budget_label TEXT,
    move_in_timeline TEXT
);

CREATE TABLE IF NOT EXISTS agent_profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    company_name TEXT,
    experience_range TEXT,
    specializations_json JSONB NOT NULL DEFAULT '[]'::jsonb
);

CREATE TABLE IF NOT EXISTS landlord_profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    property_count_range TEXT,
    property_types_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    current_agent_status TEXT,
    ownership_label TEXT
);

INSERT INTO profiles (id, user_id, full_name, phone, bio)
SELECT u.id, u.id, u.full_name, u.phone, u.bio
FROM users u
WHERE NOT EXISTS (SELECT 1 FROM profiles p WHERE p.user_id = u.id);

CREATE TABLE IF NOT EXISTS verifications (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('not_started', 'submitted', 'in_review', 'approved', 'rejected', 'expired')),
    submitted_at TIMESTAMPTZ,
    reviewed_at TIMESTAMPTZ,
    reviewed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    rejection_reason TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_verifications_user_id ON verifications(user_id);
CREATE INDEX IF NOT EXISTS idx_verifications_status ON verifications(status);

CREATE TABLE IF NOT EXISTS verification_documents (
    id UUID PRIMARY KEY,
    verification_id UUID NOT NULL REFERENCES verifications(id) ON DELETE CASCADE,
    document_type TEXT NOT NULL,
    file_url TEXT NOT NULL,
    file_key TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'uploaded' CHECK (status IN ('uploaded', 'verified', 'rejected')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_verification_documents_verification_id ON verification_documents(verification_id);

ALTER TABLE properties
    ADD COLUMN IF NOT EXISTS listing_type TEXT NOT NULL DEFAULT 'rent',
    ADD COLUMN IF NOT EXISTS property_category TEXT,
    ADD COLUMN IF NOT EXISTS bedrooms_label TEXT,
    ADD COLUMN IF NOT EXISTS bathrooms INTEGER NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS city TEXT,
    ADD COLUMN IF NOT EXISTS price_currency TEXT NOT NULL DEFAULT 'NGN',
    ADD COLUMN IF NOT EXISTS price_period TEXT NOT NULL DEFAULT 'year',
    ADD COLUMN IF NOT EXISTS available_from DATE,
    ADD COLUMN IF NOT EXISTS operational_status TEXT NOT NULL DEFAULT 'vacant';

UPDATE properties
SET city = COALESCE(NULLIF(city, ''), location)
WHERE city IS NULL;

CREATE TABLE IF NOT EXISTS property_media (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    media_type TEXT NOT NULL CHECK (media_type IN ('image', 'video')),
    file_url TEXT NOT NULL,
    file_key TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_cover BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS property_amenities (
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    amenity_key TEXT NOT NULL,
    PRIMARY KEY (property_id, amenity_key)
);

CREATE INDEX IF NOT EXISTS idx_property_media_property_id ON property_media(property_id);

CREATE TABLE IF NOT EXISTS lead_matches (
    id UUID PRIMARY KEY,
    agent_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    need_post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    matched_property_id UUID REFERENCES properties(id) ON DELETE SET NULL,
    match_score NUMERIC(5,2) NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'viewed', 'responded', 'skipped', 'expired')),
    sla_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS offers (
    id UUID PRIMARY KEY,
    need_post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    provider_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_role TEXT NOT NULL CHECK (provider_role IN ('agent', 'landlord')),
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    lead_match_id UUID REFERENCES lead_matches(id) ON DELETE SET NULL,
    offer_price_amount BIGINT NOT NULL,
    offer_price_currency TEXT NOT NULL DEFAULT 'NGN',
    offer_price_period TEXT NOT NULL DEFAULT 'year',
    move_in_date DATE,
    custom_terms TEXT,
    message TEXT NOT NULL,
    priority_send BOOLEAN NOT NULL DEFAULT FALSE,
    status TEXT NOT NULL DEFAULT 'sent' CHECK (status IN ('sent', 'viewed', 'shortlisted', 'negotiated', 'accepted', 'declined', 'expired', 'withdrawn')),
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    viewed_at TIMESTAMPTZ,
    accepted_at TIMESTAMPTZ,
    declined_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_offers_need_post_id ON offers(need_post_id);
CREATE INDEX IF NOT EXISTS idx_offers_provider_user_id ON offers(provider_user_id);

CREATE TABLE IF NOT EXISTS saved_properties (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, property_id)
);

CREATE TABLE IF NOT EXISTS bookings (
    id UUID PRIMARY KEY,
    offer_id UUID REFERENCES offers(id) ON DELETE SET NULL,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    unit_id UUID,
    seeker_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    booking_type TEXT NOT NULL CHECK (booking_type IN ('viewing', 'hold', 'move_in', 'shortlet')),
    scheduled_for TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'confirmed', 'completed', 'cancelled', 'no_show')),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS units (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    unit_code TEXT NOT NULL,
    name TEXT NOT NULL,
    unit_type TEXT,
    bedrooms_label TEXT,
    rent_amount BIGINT,
    rent_currency TEXT NOT NULL DEFAULT 'NGN',
    rent_period TEXT NOT NULL DEFAULT 'year',
    occupancy_status TEXT NOT NULL DEFAULT 'vacant' CHECK (occupancy_status IN ('vacant', 'occupied', 'reserved', 'notice', 'maintenance')),
    listing_status TEXT NOT NULL DEFAULT 'unlisted' CHECK (listing_status IN ('unlisted', 'listed', 'paused')),
    tenant_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    lease_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS leases (
    id UUID PRIMARY KEY,
    unit_id UUID NOT NULL REFERENCES units(id) ON DELETE CASCADE,
    tenant_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    landlord_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rent_amount BIGINT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'NGN',
    service_charge_amount BIGINT,
    deposit_amount BIGINT,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'renewal_due', 'expired', 'terminated')),
    agreement_file_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE units
    ADD CONSTRAINT units_lease_id_fk
    FOREIGN KEY (lease_id) REFERENCES leases(id) ON DELETE SET NULL;

CREATE TABLE IF NOT EXISTS rent_charges (
    id UUID PRIMARY KEY,
    lease_id UUID NOT NULL REFERENCES leases(id) ON DELETE CASCADE,
    unit_id UUID NOT NULL REFERENCES units(id) ON DELETE CASCADE,
    tenant_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    due_date DATE NOT NULL,
    amount BIGINT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'NGN',
    status TEXT NOT NULL DEFAULT 'due' CHECK (status IN ('due', 'paid', 'overdue', 'part_paid', 'waived')),
    paid_amount BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY,
    reference TEXT NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    booking_id UUID REFERENCES bookings(id) ON DELETE SET NULL,
    offer_id UUID REFERENCES offers(id) ON DELETE SET NULL,
    lease_id UUID REFERENCES leases(id) ON DELETE SET NULL,
    rent_charge_id UUID REFERENCES rent_charges(id) ON DELETE SET NULL,
    type TEXT NOT NULL CHECK (type IN ('charge', 'escrow_hold', 'escrow_release', 'refund', 'fee', 'payout', 'rent_collection')),
    amount BIGINT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'NGN',
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'succeeded', 'failed', 'reversed')),
    provider TEXT,
    provider_reference TEXT,
    metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payouts (
    id UUID PRIMARY KEY,
    recipient_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipient_role TEXT NOT NULL CHECK (recipient_role IN ('agent', 'landlord')),
    transaction_id UUID REFERENCES transactions(id) ON DELETE SET NULL,
    amount BIGINT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'NGN',
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'queued', 'processing', 'paid', 'failed')),
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paid_at TIMESTAMPTZ,
    failure_reason TEXT
);

CREATE TABLE IF NOT EXISTS maintenance_requests (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    unit_id UUID REFERENCES units(id) ON DELETE SET NULL,
    tenant_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    landlord_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'urgent')),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'assigned', 'in_progress', 'resolved', 'closed')),
    assigned_vendor_name TEXT,
    scheduled_for TIMESTAMPTZ,
    estimated_cost BIGINT,
    actual_cost BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS calendar_events (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    property_id UUID REFERENCES properties(id) ON DELETE SET NULL,
    unit_id UUID REFERENCES units(id) ON DELETE SET NULL,
    type TEXT NOT NULL CHECK (type IN ('booking', 'rent_followup', 'lease_review', 'inspection', 'maintenance', 'document_audit')),
    title TEXT NOT NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'pending', 'completed', 'cancelled')),
    metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS disputes (
    id UUID PRIMARY KEY,
    reference TEXT NOT NULL UNIQUE,
    reporter_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subject_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    property_id UUID REFERENCES properties(id) ON DELETE SET NULL,
    offer_id UUID REFERENCES offers(id) ON DELETE SET NULL,
    booking_id UUID REFERENCES bookings(id) ON DELETE SET NULL,
    transaction_id UUID REFERENCES transactions(id) ON DELETE SET NULL,
    type TEXT NOT NULL CHECK (type IN ('fraud', 'quality', 'cancellation', 'payment', 'impersonation', 'listing_misrepresentation')),
    priority TEXT NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'in_review', 'escalated', 'resolved', 'closed')),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    assigned_admin_id UUID REFERENCES users(id) ON DELETE SET NULL,
    resolution_summary TEXT,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS announcements (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    audience TEXT NOT NULL CHECK (audience IN ('all', 'seekers', 'agents', 'landlords', 'admins')),
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'archived')),
    published_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    data_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
