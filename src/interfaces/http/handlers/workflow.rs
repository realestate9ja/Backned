use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    domain::workflow::{
        AssignPropertyAgentInput, CertifySiteVisitInput, CreateLiveVideoSessionInput,
        CreatePropertyAgentRequestInput, CreateSiteVisitInput, CreateThreadMessageInput,
        UpdateLiveVideoSessionInput, UpdateSiteVisitInput,
    },
    interfaces::http::{
        errors::AppError,
        middleware::auth::AuthUser,
        state::AppState,
    },
};

pub async fn add_thread_message(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(response_id): Path<Uuid>,
    Json(payload): Json<CreateThreadMessageInput>,
) -> Result<(StatusCode, Json<crate::domain::workflow::RequestThreadView>), AppError> {
    let thread = state
        .workflow_use_cases
        .add_thread_message(&user, response_id, payload)
        .await?;
    Ok((StatusCode::CREATED, Json(thread)))
}

pub async fn get_thread(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(response_id): Path<Uuid>,
) -> Result<Json<crate::domain::workflow::RequestThreadView>, AppError> {
    let thread = state.workflow_use_cases.get_thread(&user, response_id).await?;
    Ok(Json(thread))
}

pub async fn create_live_video_session(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(response_id): Path<Uuid>,
    Json(payload): Json<CreateLiveVideoSessionInput>,
) -> Result<(StatusCode, Json<crate::domain::workflow::LiveVideoSession>), AppError> {
    let session = state
        .workflow_use_cases
        .create_live_video_session(&user, response_id, payload)
        .await?;
    Ok((StatusCode::CREATED, Json(session)))
}

pub async fn update_live_video_session(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<UpdateLiveVideoSessionInput>,
) -> Result<Json<crate::domain::workflow::LiveVideoSession>, AppError> {
    let session = state
        .workflow_use_cases
        .update_live_video_session(&user, session_id, payload)
        .await?;
    Ok(Json(session))
}

pub async fn get_live_video_session_access(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(session_id): Path<Uuid>,
) -> Result<Json<crate::domain::workflow::LiveVideoSessionAccess>, AppError> {
    let access = state
        .workflow_use_cases
        .get_live_video_session_access(&user, session_id)
        .await?;
    Ok(Json(access))
}

pub async fn create_site_visit(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(response_id): Path<Uuid>,
    Json(payload): Json<CreateSiteVisitInput>,
) -> Result<(StatusCode, Json<crate::domain::workflow::SiteVisitView>), AppError> {
    let site_visit = state
        .workflow_use_cases
        .create_site_visit(&user, response_id, payload)
        .await?;
    Ok((StatusCode::CREATED, Json(site_visit)))
}

pub async fn update_site_visit(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(site_visit_id): Path<Uuid>,
    Json(payload): Json<UpdateSiteVisitInput>,
) -> Result<Json<crate::domain::workflow::SiteVisitView>, AppError> {
    let site_visit = state
        .workflow_use_cases
        .update_site_visit(&user, site_visit_id, payload)
        .await?;
    Ok(Json(site_visit))
}

pub async fn certify_site_visit(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(site_visit_id): Path<Uuid>,
    Json(payload): Json<CertifySiteVisitInput>,
) -> Result<Json<crate::domain::workflow::SiteVisitView>, AppError> {
    let site_visit = state
        .workflow_use_cases
        .certify_site_visit(&user, site_visit_id, payload)
        .await?;
    Ok(Json(site_visit))
}

pub async fn create_property_agent_request(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(property_id): Path<Uuid>,
    Json(payload): Json<CreatePropertyAgentRequestInput>,
) -> Result<(StatusCode, Json<crate::domain::workflow::PropertyAgentRequest>), AppError> {
    let request = state
        .workflow_use_cases
        .create_property_agent_request(&user, property_id, payload)
        .await?;
    Ok((StatusCode::CREATED, Json(request)))
}

pub async fn assign_property_agent(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(property_id): Path<Uuid>,
    Json(payload): Json<AssignPropertyAgentInput>,
) -> Result<Json<crate::domain::properties::PropertyDetail>, AppError> {
    let property = state
        .workflow_use_cases
        .assign_property_agent(&user, property_id, payload)
        .await?;
    Ok(Json(property))
}

pub async fn verify_property(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(property_id): Path<Uuid>,
) -> Result<Json<crate::domain::properties::PropertyDetail>, AppError> {
    let property = state.workflow_use_cases.verify_property(&user, property_id).await?;
    Ok(Json(property))
}

pub async fn publish_property(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(property_id): Path<Uuid>,
) -> Result<Json<crate::domain::properties::PropertyDetail>, AppError> {
    let property = state.workflow_use_cases.publish_property(&user, property_id).await?;
    Ok(Json(property))
}
