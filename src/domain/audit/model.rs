use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub request_id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub action: String,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub success: bool,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateAuditLogInput {
    pub request_id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub action: String,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub success: bool,
    pub metadata: Value,
}
