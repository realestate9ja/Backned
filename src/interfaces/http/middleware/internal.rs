use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::interfaces::http::{errors::AppError, state::AppState};

pub struct ModerationKey;

impl<S> FromRequestParts<S> for ModerationKey
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let header = parts
            .headers
            .get("x-moderation-key")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::unauthorized("missing moderation key"))?;

        if header != app_state.moderation_api_key {
            return Err(AppError::unauthorized("invalid moderation key"));
        }

        Ok(Self)
    }
}
