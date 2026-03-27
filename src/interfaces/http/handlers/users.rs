use crate::{
    interfaces::http::{errors::AppError, state::AppState},
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

