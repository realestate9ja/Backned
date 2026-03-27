use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateResponseInput {
    pub message: String,
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
    pub created_at: DateTime<Utc>,
}

impl From<Response> for ResponseCreated {
    fn from(value: Response) -> Self {
        Self {
            id: value.id,
            post_id: value.post_id,
            responder_id: value.responder_id,
            message: value.message,
            created_at: value.created_at,
        }
    }
}

