ALTER TABLE live_video_sessions
ADD COLUMN IF NOT EXISTS provider TEXT NOT NULL DEFAULT 'livekit',
ADD COLUMN IF NOT EXISTS room_name TEXT NOT NULL DEFAULT '';

UPDATE live_video_sessions
SET room_name = CASE
    WHEN room_name = '' THEN CONCAT('verinest-live-', id::text)
    ELSE room_name
END;

ALTER TABLE live_video_sessions
DROP CONSTRAINT IF EXISTS live_video_sessions_provider_check;

ALTER TABLE live_video_sessions
ADD CONSTRAINT live_video_sessions_provider_check
CHECK (provider IN ('livekit'));

CREATE INDEX IF NOT EXISTS idx_live_video_sessions_room_name ON live_video_sessions(room_name);
