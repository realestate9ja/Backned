use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::{
    notifications::AgentPostNotificationItem,
    properties::PropertyListItem,
    responses::BuyerActiveRequest,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Buyer,
    Agent,
    Landlord,
    Admin,
}

impl UserRole {
    pub fn can_manage_properties(self) -> bool {
        matches!(self, Self::Agent | Self::Landlord)
    }

    pub fn can_moderate(self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buyer => "buyer",
            Self::Agent => "agent",
            Self::Landlord => "landlord",
            Self::Admin => "admin",
        }
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
    pub verification_status: String,
    pub verification_notes: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub quality_strikes: i32,
    pub fraud_strikes: i32,
    pub listing_restricted_until: Option<DateTime<Utc>>,
    pub is_banned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn role_label(&self) -> &'static str {
        self.role.as_str()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UserPublicView {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub role: UserRole,
    pub bio: Option<String>,
    pub average_rating: Option<f64>,
    pub review_count: i64,
    pub verification_status: String,
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
    pub average_rating: Option<f64>,
    pub review_count: i64,
    pub verification_status: String,
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
            average_rating: None,
            review_count: 0,
            verification_status: user.verification_status,
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
pub struct BootstrapAdminInput {
    pub full_name: String,
    pub email: String,
    pub password: String,
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
pub struct UpdateAgentVerificationInput {
    pub verification_status: String,
    pub verification_notes: Option<String>,
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
    pub active_requests: Vec<BuyerActiveRequest>,
    pub live_video_sessions: Vec<crate::domain::workflow::LiveVideoSession>,
    pub site_visits: Vec<crate::domain::workflow::SiteVisitView>,
}

#[derive(Debug, Serialize)]
pub struct AgentDashboard {
    pub managed_properties: Vec<PropertyListItem>,
    pub service_apartments: Vec<PropertyListItem>,
    pub unread_post_alerts: Vec<AgentPostNotificationItem>,
    pub request_threads: Vec<crate::domain::workflow::RequestThread>,
    pub live_video_sessions: Vec<crate::domain::workflow::LiveVideoSession>,
    pub site_visits: Vec<crate::domain::workflow::SiteVisitView>,
}

#[derive(Debug, Serialize)]
pub struct LandlordDashboard {
    pub owned_properties: Vec<PropertyListItem>,
    pub pending_verification_properties: Vec<PropertyListItem>,
    pub agent_requests: Vec<crate::domain::workflow::PropertyAgentRequest>,
}

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub role: UserRole,
    pub profile: UserPublicView,
    pub buyer: Option<BuyerDashboard>,
    pub agent: Option<AgentDashboard>,
    pub landlord: Option<LandlordDashboard>,
}
