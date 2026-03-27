use crate::{
    domain::users::User,
    interfaces::http::{errors::AppError, state::AppState},
};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthUser(pub User);

#[derive(Clone)]
pub struct OptionalAuthUser(pub Option<User>);

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let token = bearer_token(parts)?;
        let claims = app_state
            .jwt_service
            .decode_token(token)
            .map_err(|_| AppError::unauthorized("invalid token"))?;
        let user = app_state
            .user_repository
            .find_by_id(Uuid::from(claims.sub))
            .await?
            .ok_or_else(|| AppError::unauthorized("user not found"))?;

        Ok(Self(user))
    }
}

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Some(header_value) = parts.headers.get(axum::http::header::AUTHORIZATION) else {
            return Ok(Self(None));
        };

        let value = header_value
            .to_str()
            .map_err(|_| AppError::unauthorized("invalid authorization header"))?;

        if !value.starts_with("Bearer ") {
            return Err(AppError::unauthorized("invalid authorization scheme"));
        }

        let app_state = AppState::from_ref(state);
        let claims = app_state
            .jwt_service
            .decode_token(value.trim_start_matches("Bearer ").trim())
            .map_err(|_| AppError::unauthorized("invalid token"))?;
        let user = app_state
            .user_repository
            .find_by_id(Uuid::from(claims.sub))
            .await?
            .ok_or_else(|| AppError::unauthorized("user not found"))?;

        Ok(Self(Some(user)))
    }
}

fn bearer_token(parts: &Parts) -> Result<&str, AppError> {
    let header_value = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or_else(|| AppError::unauthorized("missing authorization header"))?;
    let value = header_value
        .to_str()
        .map_err(|_| AppError::unauthorized("invalid authorization header"))?;

    if !value.starts_with("Bearer ") {
        return Err(AppError::unauthorized("invalid authorization scheme"));
    }

    Ok(value.trim_start_matches("Bearer ").trim())
}
