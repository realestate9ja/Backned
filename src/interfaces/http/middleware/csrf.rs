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
    // The path here is just the route pattern (e.g., "/auth/login"), not "/api/v1/auth/login"
    // because the middleware is applied to the nested router
    let is_auth_exempt = path.starts_with("/auth/")
        || path == "/auth/login"
        || path == "/auth/register"
        || path == "/auth/verify-email"
        || path == "/auth/send-email-code"
        || path == "/auth/verify-email-code"
        || path == "/auth/refresh"
        || path == "/auth/send-password-reset"
        || path == "/auth/reset-password"
        || path == "/auth/me";
    
    if is_auth_exempt {
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
