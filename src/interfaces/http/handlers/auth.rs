use crate::{
    application::services::{AuditActor, AuditEvent},
    domain::users::{BootstrapAdminInput, LoginInput, RegisterUserInput},
    interfaces::http::{
        errors::AppError,
        middleware::{bootstrap::AdminBootstrapToken, request_context::RequestContext},
        state::AppState,
    },
};
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;

pub async fn register(
    State(state): State<AppState>,
    context: RequestContext,
    Json(payload): Json<RegisterUserInput>,
) -> Result<(StatusCode, Json<crate::domain::users::AuthResponse>), AppError> {
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
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn login(
    State(state): State<AppState>,
    context: RequestContext,
    Json(payload): Json<LoginInput>,
) -> Result<Json<crate::domain::users::AuthResponse>, AppError> {
    let email = payload.email.clone();
    let response = state.auth_use_cases.login(payload).await?;
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
                path: "/auth/login".to_string(),
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
    Ok(Json(response))
}

pub async fn bootstrap_admin(
    State(state): State<AppState>,
    _: AdminBootstrapToken,
    context: RequestContext,
    Json(payload): Json<BootstrapAdminInput>,
) -> Result<(StatusCode, Json<crate::domain::users::AuthResponse>), AppError> {
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
                resource_type: Some("user".to_string()),
                resource_id: Some(response.user.id),
                success: true,
                metadata: json!({}),
            },
        )
        .await
        .map_err(anyhow::Error::from)?;
    Ok((StatusCode::CREATED, Json(response)))
}
