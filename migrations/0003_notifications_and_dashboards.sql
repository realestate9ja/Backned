ALTER TABLE users
ADD COLUMN IF NOT EXISTS notifications_enabled BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS operating_city TEXT,
ADD COLUMN IF NOT EXISTS operating_state TEXT;

ALTER TABLE properties
ADD COLUMN IF NOT EXISTS is_service_apartment BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE posts
ADD COLUMN IF NOT EXISTS city TEXT NOT NULL DEFAULT '',
ADD COLUMN IF NOT EXISTS state TEXT NOT NULL DEFAULT '';

UPDATE posts
SET city = location
WHERE city = '';

UPDATE posts
SET state = location
WHERE state = '';

CREATE TABLE IF NOT EXISTS agent_post_notifications (
    id UUID PRIMARY KEY,
    agent_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    matched_city TEXT,
    matched_state TEXT,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(agent_id, post_id)
);

CREATE INDEX IF NOT EXISTS idx_users_notifications_enabled ON users(notifications_enabled);
CREATE INDEX IF NOT EXISTS idx_users_operating_city ON users(operating_city);
CREATE INDEX IF NOT EXISTS idx_users_operating_state ON users(operating_state);
CREATE INDEX IF NOT EXISTS idx_properties_service_apartment ON properties(is_service_apartment);
CREATE INDEX IF NOT EXISTS idx_posts_city ON posts(city);
CREATE INDEX IF NOT EXISTS idx_posts_state ON posts(state);
CREATE INDEX IF NOT EXISTS idx_agent_post_notifications_agent_id ON agent_post_notifications(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_post_notifications_post_id ON agent_post_notifications(post_id);
CREATE INDEX IF NOT EXISTS idx_agent_post_notifications_is_read ON agent_post_notifications(is_read);
