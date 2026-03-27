use crate::{config::Settings, domain::users::User};
use anyhow::Context;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: String,
    pub exp: usize,
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration_minutes: i64,
}

impl JwtService {
    pub fn new(settings: &Settings) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(settings.jwt_secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(settings.jwt_secret.as_bytes()),
            expiration_minutes: settings.jwt_expiration_minutes,
        }
    }

    pub fn generate_token(&self, user: &User) -> anyhow::Result<String> {
        let expiration = Utc::now() + Duration::minutes(self.expiration_minutes);
        let claims = Claims {
            sub: user.id,
            role: serde_json::to_string(&user.role)
                .context("failed to serialize role")?
                .trim_matches('"')
                .to_string(),
            exp: expiration.timestamp() as usize,
        };

        encode(&Header::default(), &claims, &self.encoding_key).context("failed to generate jwt")
    }

    pub fn decode_token(&self, token: &str) -> anyhow::Result<Claims> {
        let decoded = decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .context("failed to decode jwt")?;

        Ok(decoded.claims)
    }
}

