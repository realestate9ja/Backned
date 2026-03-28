use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AgentNotificationTarget {
    pub agent_id: Uuid,
    pub matched_city: String,
    pub matched_state: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AgentPostNotificationItem {
    pub notification_id: Uuid,
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub author_name: String,
    pub author_role: String,
    pub location: String,
    pub request_title: String,
    pub area: String,
    pub city: String,
    pub state: String,
    pub property_type: String,
    pub bedrooms: i32,
    pub min_budget: i64,
    pub max_budget: i64,
    pub pricing_preference: String,
    pub desired_features: Vec<String>,
    pub status: String,
    pub description: String,
    pub matched_city: String,
    pub matched_state: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}
