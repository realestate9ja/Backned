ALTER TABLE users
    ADD COLUMN IF NOT EXISTS username TEXT,
    ADD COLUMN IF NOT EXISTS role_change_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS role_change_reset_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS transaction_pin_hash TEXT,
    ADD COLUMN IF NOT EXISTS bank_account_linked BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS profile_completed BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS document_verified BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS trust_score INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS nationality TEXT,
    ADD COLUMN IF NOT EXISTS lga TEXT,
    ADD COLUMN IF NOT EXISTS dob TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS wallet_address TEXT,
    ADD COLUMN IF NOT EXISTS referral_code TEXT,
    ADD COLUMN IF NOT EXISTS referral_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS verification_type TEXT,
    ADD COLUMN IF NOT EXISTS verification_number TEXT,
    ADD COLUMN IF NOT EXISTS nearest_landmark TEXT,
    ADD COLUMN IF NOT EXISTS google_id TEXT,
    ADD COLUMN IF NOT EXISTS address TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username_unique
    ON users ((LOWER(username)))
    WHERE username IS NOT NULL;

CREATE TABLE IF NOT EXISTS labour_worker_profiles (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    category TEXT NOT NULL,
    experience_years INTEGER NOT NULL DEFAULT 0,
    description TEXT NOT NULL,
    hourly_rate BIGINT NOT NULL DEFAULT 0,
    daily_rate BIGINT NOT NULL DEFAULT 0,
    location_state TEXT NOT NULL,
    location_city TEXT NOT NULL,
    is_available BOOLEAN NOT NULL DEFAULT TRUE,
    skills JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_labour_worker_profiles_user_id ON labour_worker_profiles(user_id);
CREATE INDEX IF NOT EXISTS idx_labour_worker_profiles_category ON labour_worker_profiles(category);

CREATE TABLE IF NOT EXISTS labour_worker_portfolio (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    image_url TEXT NOT NULL,
    project_date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_labour_worker_portfolio_user_id ON labour_worker_portfolio(user_id);

CREATE TABLE IF NOT EXISTS labour_jobs (
    id UUID PRIMARY KEY,
    employer_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    location_state TEXT NOT NULL,
    location_city TEXT NOT NULL,
    location_address TEXT NOT NULL,
    budget BIGINT NOT NULL,
    estimated_duration_days INTEGER NOT NULL,
    partial_payment_allowed BOOLEAN NOT NULL DEFAULT FALSE,
    partial_payment_percentage NUMERIC(5,2),
    deadline TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'in_progress', 'completed', 'cancelled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_labour_jobs_employer_user_id ON labour_jobs(employer_user_id);
CREATE INDEX IF NOT EXISTS idx_labour_jobs_category ON labour_jobs(category);
CREATE INDEX IF NOT EXISTS idx_labour_jobs_status ON labour_jobs(status);

CREATE TABLE IF NOT EXISTS labour_job_applications (
    id UUID PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES labour_jobs(id) ON DELETE CASCADE,
    worker_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    proposed_rate BIGINT NOT NULL,
    estimated_completion INTEGER NOT NULL,
    cover_letter TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected', 'withdrawn')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (job_id, worker_user_id)
);

CREATE INDEX IF NOT EXISTS idx_labour_job_applications_job_id ON labour_job_applications(job_id);
CREATE INDEX IF NOT EXISTS idx_labour_job_applications_worker_user_id ON labour_job_applications(worker_user_id);

CREATE TABLE IF NOT EXISTS labour_contracts (
    id UUID PRIMARY KEY,
    job_id UUID REFERENCES labour_jobs(id) ON DELETE SET NULL,
    employer_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    worker_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    application_id UUID REFERENCES labour_job_applications(id) ON DELETE SET NULL,
    agreed_rate BIGINT NOT NULL,
    agreed_timeline INTEGER NOT NULL,
    terms TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('draft', 'active', 'completed', 'cancelled', 'disputed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_labour_contracts_job_id ON labour_contracts(job_id);
CREATE INDEX IF NOT EXISTS idx_labour_contracts_employer_user_id ON labour_contracts(employer_user_id);
CREATE INDEX IF NOT EXISTS idx_labour_contracts_worker_user_id ON labour_contracts(worker_user_id);

CREATE TABLE IF NOT EXISTS labour_job_progress (
    id UUID PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES labour_jobs(id) ON DELETE CASCADE,
    contract_id UUID REFERENCES labour_contracts(id) ON DELETE SET NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    update_text TEXT NOT NULL,
    progress_percentage INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_labour_job_progress_job_id ON labour_job_progress(job_id);

CREATE TABLE IF NOT EXISTS wallet_bank_accounts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_name TEXT NOT NULL,
    account_number TEXT NOT NULL,
    bank_code TEXT NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_wallet_bank_accounts_user_id ON wallet_bank_accounts(user_id);

CREATE TABLE IF NOT EXISTS chat_chats (
    id UUID PRIMARY KEY,
    participant_one_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    participant_two_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    job_id UUID REFERENCES labour_jobs(id) ON DELETE SET NULL,
    last_message_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (participant_one_id, participant_two_id, job_id)
);

CREATE INDEX IF NOT EXISTS idx_chat_chats_participant_one_id ON chat_chats(participant_one_id);
CREATE INDEX IF NOT EXISTS idx_chat_chats_participant_two_id ON chat_chats(participant_two_id);

CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY,
    chat_id UUID NOT NULL REFERENCES chat_chats(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    message_type TEXT NOT NULL DEFAULT 'text',
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_chat_messages_chat_id ON chat_messages(chat_id);

CREATE TABLE IF NOT EXISTS support_tickets (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,
    priority TEXT NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'in_progress', 'resolved', 'closed')),
    assigned_to UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_support_tickets_user_id ON support_tickets(user_id);

CREATE TABLE IF NOT EXISTS support_ticket_messages (
    id UUID PRIMARY KEY,
    ticket_id UUID NOT NULL REFERENCES support_tickets(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    is_internal BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_support_ticket_messages_ticket_id ON support_ticket_messages(ticket_id);

CREATE TABLE IF NOT EXISTS vendor_profiles (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    business_name TEXT NOT NULL,
    description TEXT,
    location_state TEXT NOT NULL,
    location_city TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS vendor_services (
    id UUID PRIMARY KEY,
    vendor_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,
    price BIGINT NOT NULL,
    location_state TEXT,
    location_city TEXT,
    image_urls JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('draft', 'active', 'paused', 'archived')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vendor_services_vendor_user_id ON vendor_services(vendor_user_id);

CREATE TABLE IF NOT EXISTS vendor_orders (
    id UUID PRIMARY KEY,
    service_id UUID NOT NULL REFERENCES vendor_services(id) ON DELETE CASCADE,
    buyer_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    vendor_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'paid', 'completed', 'cancelled')),
    rating INTEGER,
    review_comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_vendor_orders_buyer_user_id ON vendor_orders(buyer_user_id);
CREATE INDEX IF NOT EXISTS idx_vendor_orders_vendor_user_id ON vendor_orders(vendor_user_id);

CREATE TABLE IF NOT EXISTS vendor_inquiries (
    id UUID PRIMARY KEY,
    service_id UUID NOT NULL REFERENCES vendor_services(id) ON DELETE CASCADE,
    sender_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    vendor_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vendor_inquiries_vendor_user_id ON vendor_inquiries(vendor_user_id);
