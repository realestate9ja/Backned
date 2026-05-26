CREATE TABLE IF NOT EXISTS site_policy_settings (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton = TRUE),
    terms_version TEXT NOT NULL,
    privacy_version TEXT NOT NULL,
    effective_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    change_summary TEXT NOT NULL DEFAULT 'We updated our Terms and Privacy Policy. Please review the latest versions.',
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO site_policy_settings (singleton, terms_version, privacy_version, effective_at, change_summary)
VALUES (
    TRUE,
    '2026.05',
    '2026.05',
    NOW(),
    'We updated our Terms and Privacy Policy. Please review the latest versions before continuing to use Verinest.'
)
ON CONFLICT (singleton) DO NOTHING;

CREATE TABLE IF NOT EXISTS user_legal_acceptances (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    terms_version TEXT NOT NULL,
    privacy_version TEXT NOT NULL,
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
