use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use uuid::Uuid;

use crate::interfaces::http::errors::AppError;

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub request_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl RequestContext {
    pub fn from_parts(parts: &Parts) -> Self {
        let request_id = parts
            .headers
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| Uuid::parse_str(value).ok())
            .unwrap_or_else(Uuid::new_v4);
        let ip_address = parts
            .headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next().map(str::trim))
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| {
                parts
                    .headers
                    .get("x-real-ip")
                    .and_then(|value| value.to_str().ok())
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
            });
        let user_agent = parts
            .headers
            .get(header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);

        Self {
            request_id,
            ip_address,
            user_agent,
        }
    }
}

impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<RequestContext>()
            .cloned()
            .ok_or_else(|| AppError::internal("request context missing"))
    }
}
