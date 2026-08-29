-- Create property comments table
CREATE TABLE IF NOT EXISTS property_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL,
    CONSTRAINT content_not_empty CHECK (length(trim(content)) > 0)
);

-- Create comment replies table
CREATE TABLE IF NOT EXISTS comment_replies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    comment_id UUID NOT NULL REFERENCES property_comments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL,
    CONSTRAINT reply_content_not_empty CHECK (length(trim(content)) > 0)
);

-- Create indexes for fast queries
CREATE INDEX IF NOT EXISTS idx_property_comments_property_id ON property_comments(property_id);
CREATE INDEX IF NOT EXISTS idx_property_comments_user_id ON property_comments(user_id);
CREATE INDEX IF NOT EXISTS idx_property_comments_created_at ON property_comments(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_comment_replies_comment_id ON comment_replies(comment_id);
CREATE INDEX IF NOT EXISTS idx_comment_replies_user_id ON comment_replies(user_id);
CREATE INDEX IF NOT EXISTS idx_comment_replies_created_at ON comment_replies(created_at DESC);

-- Index for getting comments with non-deleted flag
CREATE INDEX IF NOT EXISTS idx_property_comments_active ON property_comments(property_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_comment_replies_active ON comment_replies(comment_id) WHERE deleted_at IS NULL;
