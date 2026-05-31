use axum::{
    extract::Request,
    http::{Method, HeaderMap, header::SET_COOKIE},
    middleware::Next,
    response::{IntoResponse, Response},
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
    // This includes: login, register, verify-email, send-email-code, verify-email-code,
    // refresh, send-password-reset, reset-password, logout, accept-current, and me
    if path.starts_with("/auth/") {
        return Ok(next.run(request).await);
    }
    
    if path == "/auth/legal/accept-current" {
        return Ok(next.run(request).await);
    }

    // Check if user has Authorization header (authenticated request)
    let has_auth = request.headers().contains_key("authorization");
    
    // If user is authenticated but has no CSRF cookie, generate a new one
    // This handles the case where users logged in before CSRF was implemented
    // or their CSRF cookie expired while their JWT is still valid
    if has_auth {
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
            });
        
        // If no CSRF cookie, check for CSRF header
        if let Some(csrf_token_cookie_val) = csrf_token_cookie {
            // CSRF cookie exists, validate it matches header
            let csrf_token_header = request
                .headers()
                .get("x-csrf-token")
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| AppError::forbidden("missing csrf token"))?
                .to_string();

            // Validate tokens match (timing-safe comparison)
            if !CsrfService::validate(&csrf_token_header, &csrf_token_cookie_val) {
                return Err(AppError::forbidden("csrf token mismatch"));
            }
            
            return Ok(next.run(request).await);
        } else {
            // No CSRF cookie but user is authenticated - issue a new CSRF token
            // This is a graceful fallback for existing sessions
            let csrf_token_header = request
                .headers()
                .get("x-csrf-token")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());
            
            if let Some(csrf_token) = csrf_token_header {
                // User provided a CSRF token in header but no cookie - allow it
                // This can happen with local/development testing or API clients
                let mut response = next.run(request).await;
                
                // Set the CSRF cookie for future requests
                let csrf_cookie = format!(
                    "verinest_csrf={}; Secure; SameSite=Strict; Path=/; Max-Age=604800",
                    csrf_token
                );
                if let Ok(header_value) = csrf_cookie.parse() {
                    let headers = response.headers_mut();
                    headers.insert(SET_COOKIE, header_value);
                }
                
                return Ok(response);
            } else {
                // No CSRF cookie and no CSRF header - generate a new token for next request
                // This allows the current request through but sets a cookie for future requests
                let new_csrf_token = CsrfService::generate_token();
                let mut response = next.run(request).await;
                
                let csrf_cookie = format!(
                    "verinest_csrf={}; Secure; SameSite=Strict; Path=/; Max-Age=604800",
                    new_csrf_token
                );
                if let Ok(header_value) = csrf_cookie.parse() {
                    let headers = response.headers_mut();
                    headers.insert(SET_COOKIE, header_value);
                }
                
                return Ok(response);
            }
        }
    }

    // For unauthenticated requests (non-auth endpoints), CSRF tokens are required
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
