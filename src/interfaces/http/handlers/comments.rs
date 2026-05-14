use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    application::services::CommentService,
    domain::comments::{CreateCommentInput, CreateReplyInput},
    interfaces::http::{
        errors::AppError,
        middleware::auth::AuthUser,
        state::AppState,
    },
};

pub async fn create_property_comment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(property_id): Path<Uuid>,
    Json(payload): Json<CreateCommentInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let comment_service = CommentService::new(
        state.comment_repository.clone(),
        state.user_repository.clone(),
        state.pool.clone(),
    );

    let comment = comment_service
        .create_comment(property_id, user.id, payload)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!(comment))))
}

pub async fn get_property_comments(
    State(state): State<AppState>,
    Path(property_id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);
    let offset: i64 = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let comment_service = CommentService::new(
        state.comment_repository.clone(),
        state.user_repository.clone(),
        state.pool.clone(),
    );

    let comments = comment_service
        .get_comments_with_replies(property_id, limit, offset)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?;
    let count = comment_service
        .get_comment_count(property_id)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?;

    Ok(Json(json!({
        "comments": comments,
        "total": count,
        "limit": limit,
        "offset": offset
    })))
}

pub async fn create_comment_reply(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(comment_id): Path<Uuid>,
    Json(payload): Json<CreateReplyInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Verify comment exists
    let _comment = state
        .comment_repository
        .get_comment_by_id(comment_id)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?
        .ok_or_else(|| AppError::not_found("Comment not found"))?;

    let comment_service = CommentService::new(
        state.comment_repository.clone(),
        state.user_repository.clone(),
        state.pool.clone(),
    );

    let reply = comment_service
        .create_reply(comment_id, user.id, payload)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!(reply))))
}

pub async fn update_comment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(comment_id): Path<Uuid>,
    Json(payload): Json<CreateCommentInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify ownership
    let comment = state
        .comment_repository
        .get_comment_by_id(comment_id)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?
        .ok_or_else(|| AppError::not_found("Comment not found"))?;

    if comment.user_id != user.id {
        return Err(AppError::forbidden("You can only edit your own comments"));
    }

    let updated = state
        .comment_repository
        .update_comment(comment_id, &payload.content)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?;

    Ok(Json(json!(updated)))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(comment_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Verify ownership
    let comment = state
        .comment_repository
        .get_comment_by_id(comment_id)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?
        .ok_or_else(|| AppError::not_found("Comment not found"))?;

    if comment.user_id != user.id {
        return Err(AppError::forbidden("You can only delete your own comments"));
    }

    state
        .comment_repository
        .delete_comment(comment_id)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_reply(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(reply_id): Path<Uuid>,
    Json(payload): Json<CreateReplyInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify ownership
    let reply = state
        .comment_repository
        .get_reply_by_id(reply_id)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?
        .ok_or_else(|| AppError::not_found("Reply not found"))?;

    if reply.user_id != user.id {
        return Err(AppError::forbidden("You can only edit your own replies"));
    }

    let updated = state
        .comment_repository
        .update_reply(reply_id, &payload.content)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?;

    Ok(Json(json!(updated)))
}

pub async fn delete_reply(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(reply_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Verify ownership
    let reply = state
        .comment_repository
        .get_reply_by_id(reply_id)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?
        .ok_or_else(|| AppError::not_found("Reply not found"))?;

    if reply.user_id != user.id {
        return Err(AppError::forbidden("You can only delete your own replies"));
    }

    state
        .comment_repository
        .delete_reply(reply_id)
        .await
        .map_err(|e| AppError::internal(&e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
