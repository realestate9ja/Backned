use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReviewInput {
    #[serde(alias = "reviewee_id")]
    pub reviewee_id: Uuid,
    #[serde(alias = "property_id")]
    pub property_id: Option<Uuid>,
    #[serde(alias = "response_id")]
    pub response_id: Option<Uuid>,
    pub rating: i16,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Review {
    pub id: Uuid,
    pub reviewer_id: Uuid,
    pub reviewee_id: Uuid,
    pub property_id: Option<Uuid>,
    pub response_id: Option<Uuid>,
    pub rating: i16,
    pub comment: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ReviewView {
    pub id: Uuid,
    pub reviewer_id: Uuid,
    pub reviewer_name: String,
    pub reviewee_id: Uuid,
    pub property_id: Option<Uuid>,
    pub response_id: Option<Uuid>,
    pub rating: i16,
    pub comment: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct UserReviewSummary {
    pub user_id: Uuid,
    pub average_rating: Option<f64>,
    pub review_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReportInput {
    #[serde(alias = "reported_user_id")]
    pub reported_user_id: Option<Uuid>,
    #[serde(alias = "property_id")]
    pub property_id: Option<Uuid>,
    #[serde(alias = "post_id")]
    pub post_id: Option<Uuid>,
    #[serde(alias = "response_id")]
    pub response_id: Option<Uuid>,
    #[serde(alias = "violation_type")]
    pub violation_type: String,
    pub reason: String,
    pub details: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerateReportInput {
    pub status: String,
    #[serde(alias = "review_notes")]
    pub review_notes: String,
    #[serde(alias = "property_action")]
    pub property_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Report {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub reported_user_id: Option<Uuid>,
    pub property_id: Option<Uuid>,
    pub post_id: Option<Uuid>,
    pub response_id: Option<Uuid>,
    pub violation_type: String,
    pub reason: String,
    pub details: String,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
