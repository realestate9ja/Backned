use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{domain::{notifications::AgentPostNotificationItem, posts::PostListItem, properties::PropertyListItem}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Buyer,
    Agent,
    Landlord,
}

impl UserRole {
    pub fn can_manage_properties(self) -> bool {
        matches!(self, Self::Agent | Self::Landlord)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub phone: Option<String>,
    pub bio: Option<String>,
    pub notifications_enabled: bool,
    pub operating_city: Option<String>,
    pub operating_state: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserPublicView {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub role: UserRole,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentProfile {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub bio: Option<String>,
    pub operating_city: Option<String>,
    pub operating_state: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserPublicView {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            full_name: user.full_name,
            email: user.email,
            role: user.role,
            bio: user.bio,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserInput {
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub phone: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublicView,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentNotificationSettingsInput {
    pub notifications_enabled: bool,
    pub operating_city: Option<String>,
    pub operating_state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentNotificationSettingsView {
    pub notifications_enabled: bool,
    pub operating_city: Option<String>,
    pub operating_state: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AgentNotificationRecipient {
    pub id: Uuid,
    pub operating_city: String,
    pub operating_state: String,
}

#[derive(Debug, Serialize)]
pub struct BuyerDashboard {
    pub my_posts_count: i64,
    pub recent_posts: Vec<PostListItem>,
}

#[derive(Debug, Serialize)]
pub struct AgentDashboard {
    pub managed_properties_count: i64,
    pub service_apartments_count: i64,
    pub unread_post_alerts_count: i64,
    pub recent_properties: Vec<PropertyListItem>,
    pub recent_post_alerts: Vec<AgentPostNotificationItem>,
}

#[derive(Debug, Serialize)]
pub struct LandlordDashboard {
    pub owned_properties_count: i64,
    pub assigned_agents_count: i64,
    pub recent_properties: Vec<PropertyListItem>,
}

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub role: UserRole,
    pub profile: UserPublicView,
    pub buyer: Option<BuyerDashboard>,
    pub agent: Option<AgentDashboard>,
    pub landlord: Option<LandlordDashboard>,
}
