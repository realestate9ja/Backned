use anyhow::Context;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;

#[derive(Clone)]
pub struct LiveKitService {
    server_url: String,
    api_key: String,
    token_ttl_minutes: i64,
    encoding_key: EncodingKey,
}

#[derive(Serialize)]
struct LiveKitAccessTokenClaims {
    exp: usize,
    iss: String,
    sub: String,
    nbf: usize,
    name: String,
    metadata: String,
    video: LiveKitVideoGrant,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LiveKitVideoGrant {
    room: String,
    room_join: bool,
    can_publish: bool,
    can_publish_data: bool,
    can_subscribe: bool,
}

impl LiveKitService {
    pub fn new(server_url: String, api_key: String, api_secret: String, token_ttl_minutes: i64) -> Self {
        Self {
            server_url,
            api_key,
            token_ttl_minutes,
            encoding_key: EncodingKey::from_secret(api_secret.as_bytes()),
        }
    }

    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    pub fn room_name_for_session(&self, session_id: uuid::Uuid) -> String {
        format!("verinest-live-{session_id}")
    }

    pub fn create_join_token(
        &self,
        room_name: &str,
        participant_identity: &str,
        participant_name: &str,
        metadata: &str,
        can_publish: bool,
    ) -> anyhow::Result<String> {
        let now = Utc::now();
        let claims = LiveKitAccessTokenClaims {
            exp: (now + Duration::minutes(self.token_ttl_minutes)).timestamp() as usize,
            iss: self.api_key.clone(),
            sub: participant_identity.to_string(),
            nbf: now.timestamp() as usize,
            name: participant_name.to_string(),
            metadata: metadata.to_string(),
            video: LiveKitVideoGrant {
                room: room_name.to_string(),
                room_join: true,
                can_publish,
                can_publish_data: true,
                can_subscribe: true,
            },
        };

        encode(&Header::default(), &claims, &self.encoding_key).context("failed to sign livekit token")
    }
}
