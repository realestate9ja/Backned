use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};

use crate::{
    infrastructure::rate_limit::RateLimitScope,
    interfaces::http::{errors::AppError, state::AppState},
};

pub async fn auth_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    check_limit(state, request, next, RateLimitScope::Auth).await
}

pub async fn trust_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    check_limit(state, request, next, RateLimitScope::Trust).await
}

async fn check_limit(
    state: AppState,
    request: Request<axum::body::Body>,
    next: Next,
    scope: RateLimitScope,
) -> Result<Response, AppError> {
    let path = request.uri().path().to_string();
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or("anonymous");
    let key = format!("{ip}:{path}");
    if !state.rate_limiter.check(scope, &key).await {
        return Err(AppError::too_many_requests("rate limit exceeded"));
    }

    Ok(next.run(request).await)
}
