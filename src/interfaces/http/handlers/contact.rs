use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    domain::contact::{CreateContactMessageRequest, ContactMessageResponse},
    interfaces::http::state::AppState,
};

/// Create a new contact message from the website form
pub async fn create_contact_message(
    State(state): State<AppState>,
    Json(req): Json<CreateContactMessageRequest>,
) -> Result<(StatusCode, Json<ContactMessageResponse>), (StatusCode, String)> {
    let message = state
        .contact_repository
        .create_message(&req.name, &req.email, &req.subject, &req.message)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create contact message: {}", e),
        ))?;

    Ok((
        StatusCode::CREATED,
        Json(ContactMessageResponse::from(message)),
    ))
}

/// Get all contact messages (admin only)
pub async fn get_contact_messages(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = 50i64;
    let offset = 0i64;

    let messages = state
        .contact_repository
        .get_all_messages(limit, offset)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch messages: {}", e),
        ))?;

    let total = state
        .contact_repository
        .get_total_count()
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch total count: {}", e),
        ))?;

    let unread = state
        .contact_repository
        .get_unread_count()
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch unread count: {}", e),
        ))?;

    Ok(Json(serde_json::json!({
        "messages": messages.into_iter().map(ContactMessageResponse::from).collect::<Vec<_>>(),
        "total": total,
        "unread": unread,
        "limit": limit,
        "offset": offset
    })))
}

/// Get a specific contact message by ID (admin only)
pub async fn get_contact_message(
    State(state): State<AppState>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<ContactMessageResponse>, (StatusCode, String)> {
    let message = state
        .contact_repository
        .get_message_by_id(message_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch message: {}", e),
        ))?
        .ok_or((StatusCode::NOT_FOUND, "Message not found".to_string()))?;

    // Mark as read when viewing
    state
        .contact_repository
        .mark_as_read(message_id)
        .await
        .ok();

    Ok(Json(ContactMessageResponse::from(message)))
}

/// Mark contact message as read (admin only)
pub async fn mark_message_as_read(
    State(state): State<AppState>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<ContactMessageResponse>, (StatusCode, String)> {
    let message = state
        .contact_repository
        .mark_as_read(message_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to mark message as read: {}", e),
        ))?;

    Ok(Json(ContactMessageResponse::from(message)))
}
