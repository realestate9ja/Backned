ALTER TABLE posts
ADD COLUMN IF NOT EXISTS target_agent_id UUID REFERENCES users(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS target_property_id UUID REFERENCES properties(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS target_property_title TEXT,
ADD COLUMN IF NOT EXISTS target_property_image_url TEXT,
ADD COLUMN IF NOT EXISTS target_property_location TEXT;

CREATE INDEX IF NOT EXISTS idx_posts_target_agent_id ON posts(target_agent_id);
CREATE INDEX IF NOT EXISTS idx_posts_target_property_id ON posts(target_property_id);
