use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::properties::PropertyListItem;

#[derive(Debug, Deserialize)]
pub struct CreateThreadMessageInput {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RequestThread {
    pub id: Uuid,
    pub response_id: Uuid,
    pub post_id: Uuid,
    pub buyer_id: Uuid,
    pub agent_id: Uuid,
    pub status: String,
    pub last_message_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ThreadMessage {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub sender_role: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestThreadView {
    pub thread: RequestThread,
    pub messages: Vec<ThreadMessage>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLiveVideoSessionInput {
    pub scheduled_at: Option<DateTime<Utc>>,
    pub tracking_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLiveVideoSessionInput {
    pub status: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub tracking_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct LiveVideoSession {
    pub id: Uuid,
    pub response_id: Uuid,
    pub requested_by_user_id: Uuid,
    pub buyer_id: Uuid,
    pub agent_id: Uuid,
    pub provider: String,
    pub room_name: String,
    pub status: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub tracking_notes: Option<String>,
    pub recording_saved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiveVideoSessionAccess {
    pub session: LiveVideoSession,
    pub server_url: String,
    pub room_name: String,
    pub participant_identity: String,
    pub participant_name: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSiteVisitInput {
    pub property_id: Uuid,
    pub scheduled_at: DateTime<Utc>,
    pub meeting_point: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSiteVisitInput {
    pub scheduled_at: Option<DateTime<Utc>>,
    pub meeting_point: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CertifySiteVisitInput {
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SiteVisit {
    pub id: Uuid,
    pub response_id: Uuid,
    pub buyer_id: Uuid,
    pub agent_id: Uuid,
    pub property_id: Uuid,
    pub scheduled_at: DateTime<Utc>,
    pub meeting_point: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SiteVisitCertification {
    pub id: Uuid,
    pub site_visit_id: Uuid,
    pub certified_by: Uuid,
    pub certified_at: DateTime<Utc>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SiteVisitView {
    pub site_visit: SiteVisit,
    pub property: PropertyListItem,
    pub certification: Option<SiteVisitCertification>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePropertyAgentRequestInput {
    pub requested_agent_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignPropertyAgentInput {
    pub agent_id: Uuid,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PropertyAgentRequest {
    pub id: Uuid,
    pub property_id: Uuid,
    pub landlord_id: Uuid,
    pub requested_agent_id: Option<Uuid>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ResponseWorkflowContext {
    pub response_id: Uuid,
    pub post_id: Uuid,
    pub buyer_id: Uuid,
    pub agent_id: Uuid,
}
