use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::properties::PropertyListItem;

#[derive(Debug, Deserialize)]
pub struct CreateResponseInput {
    pub message: String,
    pub property_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Response {
    pub id: Uuid,
    pub post_id: Uuid,
    pub responder_id: Uuid,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseCreated {
    pub id: Uuid,
    pub post_id: Uuid,
    pub responder_id: Uuid,
    pub message: String,
    pub properties: Vec<PropertyListItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PostResponseItem {
    pub response_id: Uuid,
    pub responder_id: Uuid,
    pub responder_name: String,
    pub responder_role: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostResponseWithProperties {
    pub response_id: Uuid,
    pub post_id: Uuid,
    pub responder_id: Uuid,
    pub responder_name: String,
    pub responder_role: String,
    pub message: String,
    pub properties: Vec<PropertyListItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeekerActiveRequest {
    pub request: crate::domain::posts::PostListItem,
    pub responses: Vec<PostResponseWithProperties>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ResponseContext {
    pub post_author_id: Uuid,
    pub responder_id: Uuid,
}
