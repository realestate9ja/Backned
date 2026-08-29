-- Create contact messages table for admin inquiries from website contact form
CREATE TABLE IF NOT EXISTS contact_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    read_at TIMESTAMPTZ NULL,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT email_not_empty CHECK (length(trim(email)) > 0),
    CONSTRAINT subject_not_empty CHECK (length(trim(subject)) > 0),
    CONSTRAINT message_not_empty CHECK (length(trim(message)) > 0)
);

-- Create indexes for fast queries
CREATE INDEX IF NOT EXISTS idx_contact_messages_created_at ON contact_messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_contact_messages_is_read ON contact_messages(is_read);
CREATE INDEX IF NOT EXISTS idx_contact_messages_email ON contact_messages(email);
