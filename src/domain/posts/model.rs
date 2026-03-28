use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreatePostInput {
    pub budget: i64,
    pub location: String,
    pub city: String,
    pub state: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct PostQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub location: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub min_budget: Option<i64>,
    pub max_budget: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub budget: i64,
    pub location: String,
    pub city: String,
    pub state: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PostListItem {
    pub id: Uuid,
    pub author_id: Uuid,
    pub author_name: String,
    pub author_role: String,
    pub budget: i64,
    pub location: String,
    pub city: String,
    pub state: String,
    pub description: String,
    pub response_count: i64,
    pub created_at: DateTime<Utc>,
}
