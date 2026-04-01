use crate::{
    domain::users::{UpdateAgentNotificationSettingsInput, UpdateAgentVerificationInput},
    interfaces::http::{
        errors::AppError,
        middleware::auth::AuthUser,
        state::AppState,
    },
    utils::pagination::PaginationParams,
};
use axum::{extract::{Path, Query, State}, Json};
use uuid::Uuid;

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::domain::users::UserPublicView>, AppError> {
    let user = state.user_use_cases.get_user(id).await?;
    Ok(Json(user))
}

pub async fn list_agents(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<crate::domain::users::AgentProfile>>, AppError> {
    let pagination = params.try_into()?;
    let agents = state.user_use_cases.list_agents(pagination).await?;
    Ok(Json(agents))
}

pub async fn update_agent_notification_settings(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<UpdateAgentNotificationSettingsInput>,
) -> Result<Json<crate::domain::users::AgentNotificationSettingsView>, AppError> {
    let settings = state
        .user_use_cases
        .update_agent_notification_settings(&user, payload)
        .await?;
    Ok(Json(settings))
}

pub async fn list_agent_post_alerts(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<crate::domain::notifications::AgentPostNotificationItem>>, AppError> {
    let alerts = state.user_use_cases.list_agent_post_alerts(&user).await?;
    Ok(Json(alerts))
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<crate::domain::users::DashboardResponse>, AppError> {
    let dashboard = state.user_use_cases.get_dashboard(&user).await?;
    Ok(Json(dashboard))
}

pub async fn update_agent_verification(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(agent_id): Path<Uuid>,
    Json(payload): Json<UpdateAgentVerificationInput>,
) -> Result<Json<crate::domain::users::UserPublicView>, AppError> {
    let agent = state
        .user_use_cases
        .update_agent_verification(&user, agent_id, payload)
        .await?;
    Ok(Json(agent))
}
