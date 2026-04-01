DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_enum
        WHERE enumlabel = 'pending_verification'
          AND enumtypid = 'property_status'::regtype
    ) THEN
        ALTER TYPE property_status ADD VALUE 'pending_verification';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_enum
        WHERE enumlabel = 'verified'
          AND enumtypid = 'property_status'::regtype
    ) THEN
        ALTER TYPE property_status ADD VALUE 'verified';
    END IF;
END $$;

ALTER TABLE properties
ADD COLUMN IF NOT EXISTS self_managed BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS verified_by UUID REFERENCES users(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS verified_at TIMESTAMPTZ;

CREATE TABLE IF NOT EXISTS property_agent_requests (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL UNIQUE REFERENCES properties(id) ON DELETE CASCADE,
    landlord_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    requested_agent_id UUID REFERENCES users(id) ON DELETE SET NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'fulfilled', 'cancelled')),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS request_threads (
    id UUID PRIMARY KEY,
    response_id UUID NOT NULL UNIQUE REFERENCES responses(id) ON DELETE CASCADE,
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    buyer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('open', 'closed')) DEFAULT 'open',
    last_message_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS thread_messages (
    id UUID PRIMARY KEY,
    thread_id UUID NOT NULL REFERENCES request_threads(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS live_video_sessions (
    id UUID PRIMARY KEY,
    response_id UUID NOT NULL REFERENCES responses(id) ON DELETE CASCADE,
    requested_by_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    buyer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (
        status IN ('requested', 'scheduled', 'live', 'completed', 'cancelled')
    ) DEFAULT 'requested',
    scheduled_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    tracking_notes TEXT,
    recording_saved BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS site_visits (
    id UUID PRIMARY KEY,
    response_id UUID NOT NULL REFERENCES responses(id) ON DELETE CASCADE,
    buyer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    scheduled_at TIMESTAMPTZ NOT NULL,
    meeting_point TEXT NOT NULL,
    status TEXT NOT NULL CHECK (
        status IN ('scheduled', 'confirmed', 'completed', 'cancelled', 'certified')
    ) DEFAULT 'scheduled',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS site_visit_certifications (
    id UUID PRIMARY KEY,
    site_visit_id UUID NOT NULL UNIQUE REFERENCES site_visits(id) ON DELETE CASCADE,
    certified_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    certified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notes TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS reviews (
    id UUID PRIMARY KEY,
    reviewer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reviewee_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    property_id UUID REFERENCES properties(id) ON DELETE SET NULL,
    response_id UUID REFERENCES responses(id) ON DELETE SET NULL,
    rating SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comment TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (reviewer_id <> reviewee_id)
);

CREATE TABLE IF NOT EXISTS reports (
    id UUID PRIMARY KEY,
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reported_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    property_id UUID REFERENCES properties(id) ON DELETE SET NULL,
    post_id UUID REFERENCES posts(id) ON DELETE SET NULL,
    response_id UUID REFERENCES responses(id) ON DELETE SET NULL,
    reason TEXT NOT NULL,
    details TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('open', 'reviewing', 'resolved')) DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_properties_status ON properties(status);
CREATE INDEX IF NOT EXISTS idx_properties_verified_by ON properties(verified_by);
CREATE INDEX IF NOT EXISTS idx_property_agent_requests_landlord_id ON property_agent_requests(landlord_id);
CREATE INDEX IF NOT EXISTS idx_property_agent_requests_requested_agent_id ON property_agent_requests(requested_agent_id);
CREATE INDEX IF NOT EXISTS idx_request_threads_buyer_id ON request_threads(buyer_id);
CREATE INDEX IF NOT EXISTS idx_request_threads_agent_id ON request_threads(agent_id);
CREATE INDEX IF NOT EXISTS idx_thread_messages_thread_id ON thread_messages(thread_id);
CREATE INDEX IF NOT EXISTS idx_live_video_sessions_response_id ON live_video_sessions(response_id);
CREATE INDEX IF NOT EXISTS idx_live_video_sessions_buyer_id ON live_video_sessions(buyer_id);
CREATE INDEX IF NOT EXISTS idx_live_video_sessions_agent_id ON live_video_sessions(agent_id);
CREATE INDEX IF NOT EXISTS idx_live_video_sessions_status ON live_video_sessions(status);
CREATE INDEX IF NOT EXISTS idx_site_visits_response_id ON site_visits(response_id);
CREATE INDEX IF NOT EXISTS idx_site_visits_buyer_id ON site_visits(buyer_id);
CREATE INDEX IF NOT EXISTS idx_site_visits_agent_id ON site_visits(agent_id);
CREATE INDEX IF NOT EXISTS idx_site_visits_property_id ON site_visits(property_id);
CREATE INDEX IF NOT EXISTS idx_site_visits_status ON site_visits(status);
CREATE INDEX IF NOT EXISTS idx_reviews_reviewee_id ON reviews(reviewee_id);
CREATE INDEX IF NOT EXISTS idx_reviews_reviewer_id ON reviews(reviewer_id);
CREATE INDEX IF NOT EXISTS idx_reports_reported_user_id ON reports(reported_user_id);
CREATE INDEX IF NOT EXISTS idx_reports_reporter_id ON reports(reporter_id);
CREATE INDEX IF NOT EXISTS idx_reports_status ON reports(status);
