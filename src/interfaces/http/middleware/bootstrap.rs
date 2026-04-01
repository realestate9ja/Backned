use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::interfaces::http::{errors::AppError, state::AppState};

pub struct AdminBootstrapToken;

impl<S> FromRequestParts<S> for AdminBootstrapToken
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let header = parts
            .headers
            .get("x-admin-bootstrap-token")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::unauthorized("missing admin bootstrap token"))?;

        if header != app_state.admin_bootstrap_token {
            return Err(AppError::unauthorized("invalid admin bootstrap token"));
        }

        Ok(Self)
    }
}
