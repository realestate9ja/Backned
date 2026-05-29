use axum::{
    extract::Request,
    http::Method,
    middleware::Next,
    response::IntoResponse,
};
use crate::interfaces::http::errors::AppError;
use crate::infrastructure::auth::CsrfService;

/// CSRF middleware that validates CSRF tokens on state-changing requests
pub async fn csrf_middleware(
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    // Skip CSRF check for GET, HEAD, OPTIONS requests (read-only, no state change)
    if matches!(&*request.method(), &Method::GET | &Method::HEAD | &Method::OPTIONS) {
        return Ok(next.run(request).await);
    }

    let path = request.uri().path();
    
    // Skip CSRF validation for auth endpoints (they don't have CSRF tokens yet)
    let auth_exempt_paths = [
        "/api/v1/auth/register",
        "/api/v1/auth/login",
        "/api/v1/auth/verify-email",
        "/api/v1/auth/send-email-code",
        "/api/v1/auth/verify-email-code",
        "/api/v1/auth/refresh",
        "/api/v1/auth/send-password-reset",
        "/api/v1/auth/reset-password",
    ];
    
    if auth_exempt_paths.contains(&path) {
        return Ok(next.run(request).await);
    }

    // Get CSRF token from request header
    let csrf_token_header = request
        .headers()
        .get("x-csrf-token")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::forbidden("missing csrf token"))?
        .to_string();

    // Get CSRF token from cookie
    let csrf_token_cookie = request
        .headers()
        .get("cookie")
        .and_then(|h| h.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find(|c| c.trim().starts_with("verinest_csrf="))
                .map(|c| {
                    c.trim()
                        .strip_prefix("verinest_csrf=")
                        .unwrap_or("")
                        .to_string()
                })
        })
        .ok_or_else(|| AppError::forbidden("missing csrf cookie"))?;

    // Validate tokens match (timing-safe comparison)
    if !CsrfService::validate(&csrf_token_header, &csrf_token_cookie) {
        return Err(AppError::forbidden("csrf token mismatch"));
    }

    Ok(next.run(request).await)
}
