use crate::{
    application::services::{AuditActor, AuditEvent},
    domain::users::{BootstrapAdminInput, LoginInput, RegisterUserInput, VerifyEmailInput, 
                    SendPasswordResetInput, ResetPasswordInput},
    interfaces::http::{
        errors::AppError,
        middleware::{bootstrap::AdminBootstrapToken, request_context::RequestContext},
        state::AppState,
    },
};
use axum::{
    Json,
    extract::{OriginalUri, Query, State},
    http::{StatusCode, HeaderMap, header::SET_COOKIE},
};
use serde_json::json;

pub async fn register(
    State(state): State<AppState>,
    context: RequestContext,
    Json(payload): Json<RegisterUserInput>,
) -> Result<(StatusCode, HeaderMap, Json<crate::domain::users::AuthResponse>), AppError> {
    let email = payload.email.clone();
    let role = serde_json::to_string(&payload.role)
        .map_err(anyhow::Error::from)?
        .trim_matches('"')
        .to_string();
    let response = state.auth_use_cases.register(payload).await?;
    state
        .audit_service
        .record(
            AuditActor {
                user_id: Some(response.user.id),
                email: Some(email),
                role: Some(role),
            },
            AuditEvent {
                request_id: context.request_id,
                action: "auth.register".to_string(),
                method: "POST".to_string(),
                path: "/auth/register".to_string(),
                status_code: StatusCode::CREATED.as_u16(),
                ip_address: context.ip_address,
                user_agent: context.user_agent,
                resource_type: Some("user".to_string()),
                resource_id: Some(response.user.id),
                success: true,
                metadata: json!({}),
            },
        )
        .await
        .map_err(anyhow::Error::from)?;
    
    // Set CSRF cookie
    let mut headers = HeaderMap::new();
    if let Some(ref csrf_token) = response.csrf_token {
        let csrf_cookie = format!(
            "verinest_csrf={}; Secure; SameSite=Strict; Path=/; Max-Age=604800",
            csrf_token
        );
        if let Ok(header_value) = csrf_cookie.parse() {
            headers.insert(SET_COOKIE, header_value);
        }
    }
    
    Ok((StatusCode::CREATED, headers, Json(response)))
}

pub async fn login(
    State(state): State<AppState>,
    context: RequestContext,
    OriginalUri(uri): OriginalUri,
    Json(payload): Json<LoginInput>,
) -> Result<(HeaderMap, Json<crate::domain::users::AuthResponse>), AppError> {
    let email = payload.email.clone();
    let path = uri.path().to_string();
    match state.auth_use_cases.login(payload).await {
        Ok(response) => {
            state
                .audit_service
                .record(
                    AuditActor {
                        user_id: Some(response.user.id),
                        email: Some(email),
                        role: Some(
                            serde_json::to_string(&response.user.role)
                                .map_err(anyhow::Error::from)?
                                .trim_matches('"')
                                .to_string(),
                        ),
                    },
                    AuditEvent {
                        request_id: context.request_id,
                        action: "auth.login".to_string(),
                        method: "POST".to_string(),
                        path,
                        status_code: StatusCode::OK.as_u16(),
                        ip_address: context.ip_address,
                        user_agent: context.user_agent,
                        resource_type: Some("user".to_string()),
                        resource_id: Some(response.user.id),
                        success: true,
                        metadata: json!({}),
                    },
                )
                .await
                .map_err(anyhow::Error::from)?;
            
            // Set CSRF and session cookies
            let mut headers = HeaderMap::new();
            if let Some(ref csrf_token) = response.csrf_token {
                let csrf_cookie = format!(
                    "verinest_csrf={}; Secure; SameSite=Strict; Path=/; Max-Age=604800",
                    csrf_token
                );
                if let Ok(header_value) = csrf_cookie.parse() {
                    headers.insert(SET_COOKIE, header_value);
                }
            }
            
            Ok((headers, Json(response)))
        }
        Err(error) => {
            state
                .audit_service
                .record(
                    AuditActor {
                        user_id: None,
                        email: Some(email),
                        role: None,
                    },
                    AuditEvent {
                        request_id: context.request_id,
                        action: "auth.login".to_string(),
                        method: "POST".to_string(),
                        path,
                        status_code: error.status.as_u16(),
                        ip_address: context.ip_address,
                        user_agent: context.user_agent,
                        resource_type: Some("user".to_string()),
                        resource_id: None,
                        success: false,
                        metadata: json!({ "message": error.message }),
                    },
                )
                .await
                .map_err(anyhow::Error::from)?;
            Err(error)
        }
    }
}

pub async fn bootstrap_admin(
    State(state): State<AppState>,
    _: AdminBootstrapToken,
    context: RequestContext,
    Json(payload): Json<BootstrapAdminInput>,
) -> Result<(StatusCode, HeaderMap, Json<crate::domain::users::AuthResponse>), AppError> {
    let email = payload.email.clone();
    let response = state.auth_use_cases.bootstrap_admin(payload).await?;
    state
        .audit_service
        .record(
            AuditActor {
                user_id: Some(response.user.id),
                email: Some(email),
                role: Some("admin".to_string()),
            },
            AuditEvent {
                request_id: context.request_id,
                action: "admin.bootstrap".to_string(),
                method: "POST".to_string(),
                path: "/admin/bootstrap".to_string(),
                status_code: StatusCode::CREATED.as_u16(),
                ip_address: context.ip_address,
                user_agent: context.user_agent,
                resource_type: Some("admin".to_string()),
                resource_id: Some(response.user.id),
                success: true,
                metadata: json!({}),
            },
        )
        .await
        .map_err(anyhow::Error::from)?;
    
    // Set CSRF cookie
    let mut headers = HeaderMap::new();
    if let Some(ref csrf_token) = response.csrf_token {
        let csrf_cookie = format!(
            "verinest_csrf={}; Secure; SameSite=Strict; Path=/; Max-Age=604800",
            csrf_token
        );
        if let Ok(header_value) = csrf_cookie.parse() {
            headers.insert(SET_COOKIE, header_value);
        }
    }
    
    Ok((StatusCode::CREATED, headers, Json(response)))
}

pub async fn verify_email(
    State(state): State<AppState>,
    context: RequestContext,
    Query(payload): Query<VerifyEmailInput>,
) -> Result<Json<crate::domain::users::UserPublicView>, AppError> {
    let response = state.auth_use_cases.verify_email(payload).await?;
    state
        .audit_service
        .record(
            AuditActor {
                user_id: Some(response.id),
                email: Some(response.email.clone()),
                role: Some(
                    serde_json::to_string(&response.role)
                        .map_err(anyhow::Error::from)?
                        .trim_matches('"')
                        .to_string(),
                ),
            },
            AuditEvent {
                request_id: context.request_id,
                action: "auth.verify_email".to_string(),
                method: "GET".to_string(),
                path: "/auth/verify-email".to_string(),
                status_code: StatusCode::OK.as_u16(),
                ip_address: context.ip_address,
                user_agent: context.user_agent,
                resource_type: Some("user".to_string()),
                resource_id: Some(response.id),
                success: true,
                metadata: json!({}),
            },
        )
        .await
        .map_err(anyhow::Error::from)?;
    Ok(Json(response))
}

pub async fn send_password_reset(
    State(state): State<AppState>,
    context: RequestContext,
    Json(payload): Json<SendPasswordResetInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    let email = payload.email.clone();
    match state.auth_use_cases.send_password_reset(payload).await {
        Ok(_) => {
            state
                .audit_service
                .record(
                    AuditActor {
                        user_id: None,
                        email: Some(email.clone()),
                        role: None,
                    },
                    AuditEvent {
                        request_id: context.request_id,
                        action: "auth.send_password_reset".to_string(),
                        method: "POST".to_string(),
                        path: "/auth/send-password-reset".to_string(),
                        status_code: StatusCode::OK.as_u16(),
                        ip_address: context.ip_address,
                        user_agent: context.user_agent,
                        resource_type: None,
                        resource_id: None,
                        success: true,
                        metadata: json!({"email": email}),
                    },
                )
                .await
                .map_err(anyhow::Error::from)?;
            Ok(Json(json!({
                "ok": true,
                "message": "Password reset email sent. Check your inbox for the link."
            })))
        }
        Err(error) => {
            state
                .audit_service
                .record(
                    AuditActor {
                        user_id: None,
                        email: Some(email),
                        role: None,
                    },
                    AuditEvent {
                        request_id: context.request_id,
                        action: "auth.send_password_reset".to_string(),
                        method: "POST".to_string(),
                        path: "/auth/send-password-reset".to_string(),
                        status_code: error.status.as_u16(),
                        ip_address: context.ip_address,
                        user_agent: context.user_agent,
                        resource_type: None,
                        resource_id: None,
                        success: false,
                        metadata: json!({"message": error.message}),
                    },
                )
                .await
                .map_err(anyhow::Error::from)?;
            Err(error)
        }
    }
}

pub async fn reset_password(
    State(state): State<AppState>,
    context: RequestContext,
    Json(payload): Json<ResetPasswordInput>,
) -> Result<Json<crate::domain::users::UserPublicView>, AppError> {
    let response = state.auth_use_cases.reset_password(payload).await?;
    state
        .audit_service
        .record(
            AuditActor {
                user_id: Some(response.id),
                email: Some(response.email.clone()),
                role: Some("user".to_string()),
            },
            AuditEvent {
                request_id: context.request_id,
                action: "auth.reset_password".to_string(),
                method: "POST".to_string(),
                path: "/auth/reset-password".to_string(),
                status_code: StatusCode::OK.as_u16(),
                ip_address: context.ip_address,
                user_agent: context.user_agent,
                resource_type: Some("user".to_string()),
                resource_id: Some(response.id),
                success: true,
                metadata: json!({}),
            },
        )
        .await
        .map_err(anyhow::Error::from)?;
    Ok(Json(response))
}
