pub mod repository;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use self::repository::ContactMessageRepository;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContactMessage {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub is_read: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateContactMessageRequest {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ContactMessageResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
}

impl From<ContactMessage> for ContactMessageResponse {
    fn from(msg: ContactMessage) -> Self {
        Self {
            id: msg.id,
            name: msg.name,
            email: msg.email,
            subject: msg.subject,
            message: msg.message,
            created_at: msg.created_at,
            is_read: msg.is_read,
        }
    }
}
