-- Disputes system migration
-- Table: disputes
CREATE TABLE disputes (
    id SERIAL PRIMARY KEY,
    booking_id INTEGER REFERENCES bookings(id) ON DELETE CASCADE,
    property_id INTEGER REFERENCES properties(id) ON DELETE CASCADE,
    raiser_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    raiser_role VARCHAR(16) NOT NULL, -- 'seeker' or 'agent'
    reason TEXT NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'open',
    verdict TEXT,
    verdict_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Table: dispute_parties (for multi-agent/party support)
CREATE TABLE dispute_parties (
    id SERIAL PRIMARY KEY,
    dispute_id INTEGER REFERENCES disputes(id) ON DELETE CASCADE,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(16) NOT NULL -- 'seeker' or 'agent'
);

-- Table: dispute_questions (admin Q&A, private per party)
CREATE TABLE dispute_questions (
    id SERIAL PRIMARY KEY,
    dispute_id INTEGER REFERENCES disputes(id) ON DELETE CASCADE,
    party_id INTEGER REFERENCES dispute_parties(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    asked_by_admin BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Table: dispute_answers (private per party, only admin sees)
CREATE TABLE dispute_answers (
    id SERIAL PRIMARY KEY,
    question_id INTEGER REFERENCES dispute_questions(id) ON DELETE CASCADE,
    answer TEXT NOT NULL,
    answered_by_party BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
