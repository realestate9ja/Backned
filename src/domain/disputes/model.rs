use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dispute {
    pub id: i32,
    pub booking_id: i32,
    pub property_id: i32,
    pub raiser_id: Option<i32>,
    pub raiser_role: String, // "seeker" or "agent"
    pub reason: String,
    pub status: String, // "open", "closed", etc.
    pub verdict: Option<String>,
    pub verdict_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisputeParty {
    pub id: i32,
    pub dispute_id: i32,
    pub user_id: i32,
    pub role: String, // "seeker" or "agent"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisputeQuestion {
    pub id: i32,
    pub dispute_id: i32,
    pub party_id: i32,
    pub question: String,
    pub asked_by_admin: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisputeAnswer {
    pub id: i32,
    pub question_id: i32,
    pub answer: String,
    pub answered_by_party: bool,
    pub created_at: DateTime<Utc>,
}
