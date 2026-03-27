use crate::{
    domain::{posts::{CreatePostInput, PostQuery}, responses::CreateResponseInput},
    interfaces::http::{
        errors::AppError,
        middleware::auth::AuthUser,
        state::AppState,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct CreatedIdResponse {
    pub id: Uuid,
}

pub async fn create_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreatePostInput>,
) -> Result<(StatusCode, Json<CreatedIdResponse>), AppError> {
    let id = state.post_use_cases.create_post(&user, payload).await?;
    Ok((StatusCode::CREATED, Json(CreatedIdResponse { id })))
}

pub async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<PostQuery>,
) -> Result<Json<Vec<crate::domain::posts::PostListItem>>, AppError> {
    let posts = state.post_use_cases.list_posts(query).await?;
    Ok(Json(posts))
}

pub async fn respond_to_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(post_id): Path<Uuid>,
    Json(payload): Json<CreateResponseInput>,
) -> Result<(StatusCode, Json<crate::domain::responses::ResponseCreated>), AppError> {
    let response = state.post_use_cases.respond(&user, post_id, payload).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

