use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap, header::SET_COOKIE},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

use crate::infrastructure::auth::PasswordService;
use crate::infrastructure::email::service::{
    header_asset_url, kyc_header_asset, HEADER_LEAD_ALERT, HEADER_NEW_MATCH,
    HEADER_SECURITY_DARK,
};
use crate::application::services::{AuditActor, AuditEvent};

use crate::{
    domain::{
        posts::{CreatePostInput, PostListItem, PostQuery},
        properties::{PropertyListItem, PropertyQuery},
        users::{
            AuthResponse, RegisterUserInput, SendEmailCodeInput, UpdateAgentVerificationInput,
            User, UserPublicView, UserRole, VerifyEmailCodeInput,
        },
    },
    interfaces::http::{errors::AppError, middleware::auth::AuthUser, state::AppState},
    utils::pagination::{Pagination, PaginationParams},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordInput {
    pub code: String,
    pub new_password: String,
    pub new_password_confirm: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordWithOtpInput {
    pub code: String,
    pub new_password: String,
    pub new_password_confirm: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiRegisterInput {
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
    pub bio: Option<String>,
    pub accepted_terms_version: String,
    pub accepted_privacy_version: String,
}

#[derive(Debug, Deserialize)]
pub struct SelectRoleInput {
    pub role: UserRole,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingProfileInput {
    pub role: Option<UserRole>,
    pub phone: Option<String>,
    pub city: Option<String>,
    pub operating_state: Option<String>,
    #[serde(alias = "avatar_url")]
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub preferred_city: Option<String>,
    pub preferred_accommodation_type: Option<String>,
    pub preferred_budget_label: Option<String>,
    pub move_in_timeline: Option<String>,
    pub company_name: Option<String>,
    pub experience_range: Option<String>,
    pub specializations: Option<Vec<String>>,
    pub property_count_range: Option<String>,
    pub property_types: Option<Vec<String>>,
    pub current_agent_status: Option<String>,
    pub ownership_label: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVerificationInput {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationDocumentInput {
    pub document_type: String,
    pub file_url: String,
    pub file_key: String,
    pub mime_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOfferInput {
    pub need_post_id: Uuid,
    pub property_id: Uuid,
    pub lead_match_id: Option<Uuid>,
    pub offer_price_amount: i64,
    pub offer_price_currency: Option<String>,
    pub offer_price_period: Option<String>,
    pub move_in_date: Option<NaiveDate>,
    pub custom_terms: Option<String>,
    pub message: String,
    pub priority_send: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePropertyInput {
    pub property_id: Uuid,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookingInput {
    pub offer_id: Uuid,
    pub property_id: Uuid,
    pub booking_type: String,
    pub scheduled_for: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBookingInput {
    pub scheduled_for: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmBookingOutcomeInput {
    pub outcome: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookingDisputeInput {
    pub dispute_type: String,
    pub title: String,
    pub description: String,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOfferStatusInput {
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCreateReviewInput {
    pub reviewee_id: Uuid,
    pub property_id: Option<Uuid>,
    pub rating: i16,
    pub comment: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadPresignInput {
    pub category: String,
    pub filename: String,
    pub content_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgentPropertyInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<i64>,
    pub price_change_reason: Option<String>,
    pub location: Option<String>,
    pub exact_address: Option<String>,
    pub images: Option<Vec<String>>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub listing_type: Option<String>,
    pub status: Option<String>,
    pub available_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewPropertyChangeRequestInput {
    pub review_note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUnitInput {
    pub property_id: Uuid,
    pub unit_code: String,
    pub name: String,
    pub unit_type: Option<String>,
    pub bedrooms_label: Option<String>,
    pub rent_amount: Option<i64>,
    pub rent_currency: Option<String>,
    pub rent_period: Option<String>,
    pub occupancy_status: Option<String>,
    pub listing_status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMaintenanceRequestInput {
    pub property_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub estimated_cost: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAnnouncementInput {
    pub title: String,
    pub body: String,
    pub audience: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementView {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub audience: String,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminDeletePropertyInput {
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthMeResponse {
    pub user: UserPublicView,
    pub profile: Option<ProfileView>,
    pub role_profile: Option<Value>,
    pub verification: Option<VerificationView>,
    pub verification_documents: Vec<VerificationDocumentView>,
    pub liveness_completed: bool,
    pub policy_metadata: PolicyMetadataView,
    pub policy_acceptance: PolicyAcceptanceView,
}

#[derive(Debug, Serialize, FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicyMetadataView {
    pub terms_version: String,
    pub privacy_version: String,
    pub effective_at: DateTime<Utc>,
    pub change_summary: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicyAcceptanceView {
    pub terms_version_accepted: Option<String>,
    pub privacy_version_accepted: Option<String>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub requires_reacceptance: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePolicyMetadataInput {
    pub terms_version: String,
    pub privacy_version: String,
    pub effective_at: Option<DateTime<Utc>>,
    pub change_summary: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProfileView {
    pub id: Uuid,
    pub user_id: Uuid,
    pub full_name: String,
    pub phone: Option<String>,
    pub city: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub onboarding_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ActivityItem {
    pub action: String,
    pub resource_type: Option<String>,
    pub method: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct VerificationView {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>,
    pub rejection_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct VerificationDocumentView {
    pub id: Uuid,
    pub verification_id: Uuid,
    pub document_type: String,
    pub file_url: String,
    pub file_key: String,
    pub mime_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationDetailResponse {
    pub verification: VerificationView,
    pub documents: Vec<VerificationDocumentView>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgentLeadView {
    pub id: Uuid,
    pub need_post_id: Uuid,
    pub matched_property_id: Option<Uuid>,
    #[sqlx(default)]
    pub target_property_id: Option<Uuid>,
    #[sqlx(default)]
    pub target_property_title: Option<String>,
    #[sqlx(default)]
    pub target_property_image_url: Option<String>,
    #[sqlx(default)]
    pub target_property_location: Option<String>,
    pub match_score: f64,
    pub status: String,
    pub sla_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub request_title: String,
    pub location: String,
    pub property_type: String,
    pub urgency: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OfferView {
    pub id: Uuid,
    pub need_post_id: Uuid,
    pub provider_user_id: Uuid,
    pub provider_role: String,
    pub property_id: Uuid,
    pub lead_match_id: Option<Uuid>,
    pub offer_price_amount: i64,
    pub offer_price_currency: String,
    pub offer_price_period: String,
    pub move_in_date: Option<NaiveDate>,
    pub custom_terms: Option<String>,
    pub message: String,
    pub priority_send: bool,
    pub status: String,
    pub sent_at: DateTime<Utc>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub declined_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(default)]
    pub property_title: Option<String>,
    #[sqlx(default)]
    pub property_location: Option<String>,
    #[sqlx(default)]
    pub property_images: Option<Vec<String>>,
    #[sqlx(default)]
    pub provider_name: Option<String>,
    #[sqlx(default)]
    pub provider_phone: Option<String>,
    #[sqlx(default)]
    pub provider_image_url: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SavedPropertyView {
    pub id: Uuid,
    pub user_id: Uuid,
    pub property_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BookingView {
    pub id: Uuid,
    pub offer_id: Option<Uuid>,
    pub property_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub seeker_user_id: Uuid,
    pub provider_user_id: Uuid,
    pub booking_type: String,
    pub scheduled_for: DateTime<Utc>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(default)]
    pub confirmed_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub property_title: Option<String>,
    #[sqlx(default)]
    pub property_location: Option<String>,
    #[sqlx(default)]
    pub provider_name: Option<String>,
    #[sqlx(default)]
    pub seeker_name: Option<String>,
    #[sqlx(default)]
    pub provider_phone: Option<String>,
    #[sqlx(default)]
    pub provider_avatar_url: Option<String>,
    #[sqlx(default)]
    pub property_listing_type: Option<String>,
    #[sqlx(default)]
    pub seeker_outcome: Option<String>,
    #[sqlx(default)]
    pub seeker_outcome_note: Option<String>,
    #[sqlx(default)]
    pub seeker_outcome_confirmed_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub provider_outcome: Option<String>,
    #[sqlx(default)]
    pub provider_outcome_note: Option<String>,
    #[sqlx(default)]
    pub provider_outcome_confirmed_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub outcome_resolution: Option<String>,
    pub outcome_follow_up_required: bool,
    pub listing_outcome_applied: bool,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PropertyReviewView {
    pub id: Uuid,
    pub reviewer_id: Uuid,
    pub reviewer_name: String,
    pub reviewee_id: Uuid,
    pub reviewee_name: String,
    pub property_id: Option<Uuid>,
    pub rating: i16,
    pub comment: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UnitView {
    pub id: Uuid,
    pub property_id: Uuid,
    pub unit_code: String,
    pub name: String,
    pub unit_type: Option<String>,
    pub bedrooms_label: Option<String>,
    pub rent_amount: Option<i64>,
    pub rent_currency: String,
    pub rent_period: String,
    pub occupancy_status: String,
    pub listing_status: String,
    pub tenant_user_id: Option<Uuid>,
    pub lease_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CollectionView {
    pub id: Uuid,
    pub lease_id: Uuid,
    pub unit_id: Uuid,
    pub tenant_user_id: Uuid,
    pub due_date: NaiveDate,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub paid_amount: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PayoutView {
    pub id: Uuid,
    pub recipient_user_id: Uuid,
    pub recipient_role: String,
    pub transaction_id: Option<Uuid>,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub requested_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceView {
    pub id: Uuid,
    pub property_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub tenant_user_id: Option<Uuid>,
    pub landlord_user_id: Uuid,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub status: String,
    pub assigned_vendor_name: Option<String>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub estimated_cost: Option<i64>,
    pub actual_cost: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventView {
    pub id: Uuid,
    pub user_id: Uuid,
    pub property_id: Option<Uuid>,
    pub unit_id: Option<Uuid>,
    pub event_type: String,
    pub title: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub status: String,
    pub metadata_json: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminOverviewMetrics {
    pub total_properties: i64,
    pub active_users: i64,
    pub monthly_revenue: i64,
    pub open_disputes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminNeedTrendPoint {
    pub month: String,
    pub needs_created: i64,
    pub needs_answered: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminNeedAnalytics {
    pub total_needs: i64,
    pub answered_needs: i64,
    pub open_needs: i64,
    pub response_count: i64,
    pub answer_rate: f64,
    pub monthly_trend: Vec<AdminNeedTrendPoint>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminVerificationQueueItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub user_role: String,
    pub status: String,
    pub property_count: i64,
    pub document_types: Vec<String>,
    pub documents: Value,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

async fn insert_notification(
    pool: &PgPool,
    user_id: Uuid,
    kind: &str,
    title: &str,
    body: &str,
    action_url: Option<&str>,
    extra_data: Value,
) -> Result<(), AppError> {
    let mut data = extra_data;
    if let Some(url) = action_url {
        data["actionUrl"] = json!(url);
    }

    sqlx::query(
        "INSERT INTO notifications (id, user_id, type, title, body, data_json) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(kind)
    .bind(title)
    .bind(body)
    .bind(data)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<ApiRegisterInput>,
) -> Result<(StatusCode, HeaderMap, Json<AuthResponse>), AppError> {
    let policy_metadata = fetch_policy_metadata(&state.pool).await?;
    if payload.accepted_terms_version.trim() != policy_metadata.terms_version
        || payload.accepted_privacy_version.trim() != policy_metadata.privacy_version
    {
        return Err(AppError::bad_request(
            "please accept the latest Terms and Privacy Policy before continuing",
        ));
    }

    let response = state
        .auth_use_cases
        .register(RegisterUserInput {
            full_name: payload.full_name,
            email: payload.email,
            password: payload.password,
            role: UserRole::Unassigned,
            phone: payload.phone,
            bio: payload.bio,
        })
        .await?;

    upsert_user_policy_acceptance(
        &state.pool,
        response.user.id,
        &policy_metadata.terms_version,
        &policy_metadata.privacy_version,
    )
    .await?;

    // Set CSRF cookie
    let mut headers = HeaderMap::new();
    if let Some(ref csrf_token) = response.csrf_token {
        let csrf_cookie = format!(
            "verinest_csrf={}; Secure; SameSite=Strict; Path=/; Max-Age=604800",
            csrf_token
        );
        if let Ok(header_value) = csrf_cookie.parse() {
            headers.insert(SET_COOKIE, header_value);
        }
    }

    Ok((StatusCode::CREATED, headers, Json(response)))
}

pub async fn send_email_code(
    State(state): State<AppState>,
    Json(payload): Json<SendEmailCodeInput>,
) -> Result<Json<crate::application::services::ValueAck>, AppError> {
    Ok(Json(state.auth_use_cases.send_email_code(payload).await?))
}

pub async fn send_password_change_otp(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<crate::application::services::ValueAck>, AppError> {
    let code = state
        .user_repository
        .create_email_verification_code(user.id, &user.email, "password_change")
        .await?;
    let email = state
        .mail_service
        .verification_code_email(user.email.clone(), &user.full_name, &code, &header_asset_url(HEADER_SECURITY_DARK));
    state.mail_service.send(email).await?;

    Ok(Json(crate::application::services::ValueAck {
        ok: true,
        expires_in_seconds: 600,
        code_length: 5,
    }))
}

pub async fn verify_email_code(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailCodeInput>,
) -> Result<Json<UserPublicView>, AppError> {
    Ok(Json(state.auth_use_cases.verify_email_code(payload).await?))
}

pub async fn me(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<AuthMeResponse>, AppError> {
    let policy_metadata = fetch_policy_metadata(&state.pool).await?;
    let policy_acceptance = fetch_policy_acceptance(&state.pool, user.id, &policy_metadata).await?;
    let profile = fetch_profile(&state.pool, user.id).await?;
    let role_profile = fetch_role_profile(&state.pool, &user).await?;
    let verification = fetch_latest_verification(&state.pool, user.id).await?;
    let verification_documents = if let Some(item) = &verification {
        fetch_verification_documents(&state.pool, item.id).await?
    } else {
        Vec::new()
    };
    let liveness_completed = verification
        .as_ref()
        .map(|item| {
            matches!(
                item.status.as_str(),
                "submitted" | "pending" | "in_review" | "approved" | "verified"
            ) && verification_documents
                .iter()
                .any(|doc| doc.document_type == "selfie")
        })
        .unwrap_or(false);
    Ok(Json(AuthMeResponse {
        user: UserPublicView::from(user),
        profile,
        role_profile,
        verification,
        verification_documents,
        liveness_completed,
        policy_metadata,
        policy_acceptance,
    }))
}

pub async fn get_policy_metadata_public(
    State(state): State<AppState>,
) -> Result<Json<PolicyMetadataView>, AppError> {
    Ok(Json(fetch_policy_metadata(&state.pool).await?))
}

pub async fn accept_current_policies(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<PolicyAcceptanceView>, AppError> {
    let policy_metadata = fetch_policy_metadata(&state.pool).await?;
    upsert_user_policy_acceptance(
        &state.pool,
        user.id,
        &policy_metadata.terms_version,
        &policy_metadata.privacy_version,
    )
    .await?;

    Ok(Json(fetch_policy_acceptance(&state.pool, user.id, &policy_metadata).await?))
}

pub async fn get_admin_policy_metadata(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<PolicyMetadataView>, AppError> {
    if !user.role.can_moderate() {
        return Err(AppError::forbidden("only admins can manage policy metadata"));
    }

    Ok(Json(fetch_policy_metadata(&state.pool).await?))
}

pub async fn update_admin_policy_metadata(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<UpdatePolicyMetadataInput>,
) -> Result<Json<PolicyMetadataView>, AppError> {
    if !user.role.can_moderate() {
        return Err(AppError::forbidden("only admins can manage policy metadata"));
    }

    if payload.terms_version.trim().is_empty() || payload.privacy_version.trim().is_empty() {
        return Err(AppError::bad_request("policy versions are required"));
    }
    if payload.change_summary.trim().is_empty() {
        return Err(AppError::bad_request("change summary is required"));
    }

    sqlx::query(
        r#"
        INSERT INTO site_policy_settings (
            singleton, terms_version, privacy_version, effective_at, change_summary, updated_by, updated_at
        )
        VALUES (TRUE, $1, $2, COALESCE($3, NOW()), $4, $5, NOW())
        ON CONFLICT (singleton) DO UPDATE
        SET terms_version = EXCLUDED.terms_version,
            privacy_version = EXCLUDED.privacy_version,
            effective_at = EXCLUDED.effective_at,
            change_summary = EXCLUDED.change_summary,
            updated_by = EXCLUDED.updated_by,
            updated_at = NOW()
        "#,
    )
    .bind(payload.terms_version.trim())
    .bind(payload.privacy_version.trim())
    .bind(payload.effective_at)
    .bind(payload.change_summary.trim())
    .bind(user.id)
    .execute(&state.pool)
    .await?;

    Ok(Json(fetch_policy_metadata(&state.pool).await?))
}

pub async fn get_activity(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<ActivityItem>>, AppError> {
    let limit = query
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(10);

    let activity = sqlx::query_as::<_, ActivityItem>(
        r#"
        SELECT 
            action,
            resource_type,
            method,
            created_at as timestamp
        FROM audit_logs
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(user.id)
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(activity))
}

pub async fn select_onboarding_role(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<SelectRoleInput>,
) -> Result<Json<AuthMeResponse>, AppError> {
    if user.role != UserRole::Unassigned {
        return Err(AppError::forbidden("role has already been assigned"));
    }
    if !matches!(
        payload.role,
        UserRole::Seeker | UserRole::Agent | UserRole::Landlord
    ) {
        return Err(AppError::bad_request("invalid onboarding role"));
    }

    let updated = state
        .user_repository
        .update_role(user.id, payload.role)
        .await?
        .ok_or_else(|| AppError::not_found("user not found"))?;
    let profile = fetch_profile(&state.pool, user.id).await?;
    let role_profile = fetch_role_profile(&state.pool, &updated).await?;
    let verification = fetch_latest_verification(&state.pool, user.id).await?;
    let verification_documents = if let Some(item) = &verification {
        fetch_verification_documents(&state.pool, item.id).await?
    } else {
        Vec::new()
    };
    let liveness_completed = verification
        .as_ref()
        .map(|item| {
            matches!(
                item.status.as_str(),
                "submitted" | "pending" | "in_review" | "approved" | "verified"
            ) && verification_documents
                .iter()
                .any(|doc| doc.document_type == "selfie")
        })
        .unwrap_or(false);

    Ok(Json(AuthMeResponse {
        user: UserPublicView::from(updated),
        profile,
        role_profile,
        verification,
        verification_documents,
        liveness_completed,
        policy_metadata: PolicyMetadataView {
            terms_version: "1.0".to_string(),
            privacy_version: "1.0".to_string(),
            effective_at: Utc::now(),
            change_summary: "".to_string(),
            updated_at: Utc::now(),
        },
        policy_acceptance: PolicyAcceptanceView {
            terms_version_accepted: None,
            privacy_version_accepted: None,
            accepted_at: None,
            requires_reacceptance: false,
        },
    }))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.auth_use_cases.refresh(&payload.refresh_token).await?;
    Ok(Json(response))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<StatusCode, AppError> {
    state.auth_use_cases.logout(&payload.refresh_token).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_password(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<UpdatePasswordInput>,
) -> Result<StatusCode, AppError> {
    if payload.new_password != payload.new_password_confirm {
        return Err(AppError::bad_request(
            "password confirmation does not match",
        ));
    }

    crate::utils::validation::validate_password(&payload.new_password)?;

    let has_valid_code = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM email_verification_codes
            WHERE user_id = $1
              AND email = $2
              AND purpose = 'password_change'
              AND code = $3
              AND used_at IS NULL
              AND expires_at > NOW()
        )
        "#,
    )
    .bind(user.id)
    .bind(user.email.to_lowercase())
    .bind(payload.code.trim())
    .fetch_one(&state.pool)
    .await?;

    if !has_valid_code {
        return Err(AppError::bad_request("invalid or expired verification code"));
    }

    let hash = PasswordService.hash_password(&payload.new_password)?;
    sqlx::query("UPDATE users SET password_hash = $2, updated_at = NOW() WHERE id = $1")
        .bind(user.id)
        .bind(hash)
        .execute(&state.pool)
        .await?;

    state
        .user_repository
        .mark_email_verification_code_used(&user.email, payload.code.trim())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_password_with_otp(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<UpdatePasswordWithOtpInput>,
) -> Result<StatusCode, AppError> {
    if payload.new_password != payload.new_password_confirm {
        return Err(AppError::bad_request(
            "password confirmation does not match",
        ));
    }

    crate::utils::validation::validate_password(&payload.new_password)?;

    let has_valid_code = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM email_verification_codes
            WHERE user_id = $1
              AND email = $2
              AND purpose = 'password_change'
              AND code = $3
              AND used_at IS NULL
              AND expires_at > NOW()
        )
        "#,
    )
    .bind(user.id)
    .bind(user.email.to_lowercase())
    .bind(payload.code.trim())
    .fetch_one(&state.pool)
    .await?;

    if !has_valid_code {
        return Err(AppError::bad_request("invalid or expired verification code"));
    }

    let hash = PasswordService.hash_password(&payload.new_password)?;
    sqlx::query("UPDATE users SET password_hash = $2, updated_at = NOW() WHERE id = $1")
        .bind(user.id)
        .bind(hash)
        .execute(&state.pool)
        .await?;

    state
        .user_repository
        .mark_email_verification_code_used(&user.email, payload.code.trim())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn upsert_onboarding_profile(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<OnboardingProfileInput>,
) -> Result<Json<AuthMeResponse>, AppError> {
    if let Some(role) = payload.role
        && role != user.role
    {
        return Err(AppError::forbidden("role mismatch"));
    }
    sqlx::query(
        r#"
        INSERT INTO profiles (id, user_id, full_name, phone, city, avatar_url, bio, onboarding_completed)
        VALUES ($1, $1, $2, $3, $4, $5, $6, TRUE)
        ON CONFLICT (user_id) DO UPDATE
        SET full_name = EXCLUDED.full_name,
            phone = EXCLUDED.phone,
            city = EXCLUDED.city,
            avatar_url = EXCLUDED.avatar_url,
            bio = EXCLUDED.bio,
            onboarding_completed = TRUE,
            updated_at = NOW()
        "#,
    )
    .bind(user.id)
    .bind(&user.full_name)
    .bind(payload.phone.as_ref().map(|value| value.trim().to_string()))
    .bind(payload.city.as_ref().map(|value| value.trim().to_string()))
    .bind(payload.avatar_url.as_ref().map(|value| value.trim().to_string()))
    .bind(payload.bio.as_ref().map(|value| value.trim().to_string()))
    .execute(&state.pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE users
        SET phone = COALESCE($2, phone),
            bio = COALESCE($3, bio),
            operating_city = CASE WHEN role = 'agent' THEN COALESCE($4, operating_city) ELSE operating_city END,
            operating_state = CASE WHEN role = 'agent' THEN COALESCE($5, operating_state) ELSE operating_state END,
            notifications_enabled = CASE
                WHEN role = 'agent' AND COALESCE($4, '') <> '' AND COALESCE($5, '') <> '' THEN TRUE
                ELSE notifications_enabled
            END,
            wallet_address = COALESCE($6, wallet_address),
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(user.id)
    .bind(payload.phone.as_ref().map(|value| value.trim().to_string()))
    .bind(payload.bio.as_ref().map(|value| value.trim().to_string()))
    .bind(payload.city.as_ref().map(|value| value.trim().to_string()))
    .bind(payload.operating_state.as_ref().map(|value| value.trim().to_string()))
    .bind(payload.avatar_url.as_ref().map(|value| value.trim().to_string()))
    .execute(&state.pool)
    .await?;

    match user.role {
        UserRole::Seeker => {
            sqlx::query(
                r#"
                INSERT INTO seeker_profiles (
                    user_id, preferred_city, preferred_accommodation_type, preferred_budget_label, move_in_timeline
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (user_id) DO UPDATE
                SET preferred_city = EXCLUDED.preferred_city,
                    preferred_accommodation_type = EXCLUDED.preferred_accommodation_type,
                    preferred_budget_label = EXCLUDED.preferred_budget_label,
                    move_in_timeline = EXCLUDED.move_in_timeline
                "#,
            )
            .bind(user.id)
            .bind(payload.preferred_city)
            .bind(payload.preferred_accommodation_type)
            .bind(payload.preferred_budget_label)
            .bind(payload.move_in_timeline)
            .execute(&state.pool)
            .await?;
        }
        UserRole::Agent => {
            sqlx::query(
                r#"
                INSERT INTO agent_profiles (user_id, company_name, experience_range, specializations_json)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (user_id) DO UPDATE
                SET company_name = EXCLUDED.company_name,
                    experience_range = EXCLUDED.experience_range,
                    specializations_json = EXCLUDED.specializations_json
                "#,
            )
            .bind(user.id)
            .bind(payload.company_name)
            .bind(payload.experience_range)
            .bind(json!(payload.specializations.unwrap_or_default()))
            .execute(&state.pool)
            .await?;
        }
        UserRole::Landlord => {
            sqlx::query(
                r#"
                INSERT INTO landlord_profiles (
                    user_id, property_count_range, property_types_json, current_agent_status, ownership_label
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (user_id) DO UPDATE
                SET property_count_range = EXCLUDED.property_count_range,
                    property_types_json = EXCLUDED.property_types_json,
                    current_agent_status = EXCLUDED.current_agent_status,
                    ownership_label = EXCLUDED.ownership_label
                "#,
            )
            .bind(user.id)
            .bind(payload.property_count_range)
            .bind(json!(payload.property_types.unwrap_or_default()))
            .bind(payload.current_agent_status)
            .bind(payload.ownership_label)
            .execute(&state.pool)
            .await?;
        }
        UserRole::Admin | UserRole::Unassigned => {}
    }

    let refreshed_user = state
        .user_repository
        .find_by_id(user.id)
        .await?
        .ok_or_else(|| AppError::not_found("user not found"))?;
    let profile = fetch_profile(&state.pool, user.id).await?;
    let role_profile = fetch_role_profile(&state.pool, &refreshed_user).await?;
    let verification = fetch_latest_verification(&state.pool, user.id).await?;
    let verification_documents = if let Some(item) = &verification {
        fetch_verification_documents(&state.pool, item.id).await?
    } else {
        Vec::new()
    };
    let liveness_completed = verification
        .as_ref()
        .map(|item| {
            matches!(
                item.status.as_str(),
                "submitted" | "pending" | "in_review" | "approved" | "verified"
            ) && verification_documents
                .iter()
                .any(|doc| doc.document_type == "selfie")
        })
        .unwrap_or(false);

    Ok(Json(AuthMeResponse {
        user: UserPublicView::from(refreshed_user),
        profile,
        role_profile,
        verification,
        verification_documents,
        liveness_completed,
        policy_metadata: PolicyMetadataView {
            terms_version: "1.0".to_string(),
            privacy_version: "1.0".to_string(),
            effective_at: Utc::now(),
            change_summary: "".to_string(),
            updated_at: Utc::now(),
        },
        policy_acceptance: PolicyAcceptanceView {
            terms_version_accepted: None,
            privacy_version_accepted: None,
            accepted_at: None,
            requires_reacceptance: false,
        },
    }))
}

pub async fn update_user_avatar(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<Value>,
) -> Result<Json<AuthMeResponse>, AppError> {
    let avatar_url = payload
        .get("avatarUrl")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    sqlx::query(
        r#"
        INSERT INTO profiles (id, user_id, full_name, avatar_url, onboarding_completed)
        VALUES ($1, $1, $2, $3, FALSE)
        ON CONFLICT (user_id) DO UPDATE
        SET avatar_url = EXCLUDED.avatar_url,
            updated_at = NOW()
        "#,
    )
    .bind(user.id)
    .bind(&user.full_name)
    .bind(&avatar_url)
    .execute(&state.pool)
    .await?;

    sqlx::query(
        "UPDATE users SET wallet_address = $2, updated_at = NOW() WHERE id = $1",
    )
    .bind(user.id)
    .bind(&avatar_url)
    .execute(&state.pool)
    .await?;

    let profile = fetch_profile(&state.pool, user.id).await?;
    let refreshed_user = state
        .user_repository
        .find_by_id(user.id)
        .await?
        .ok_or_else(|| AppError::not_found("user not found"))?;
    let role_profile = fetch_role_profile(&state.pool, &refreshed_user).await?;
    let verification = fetch_latest_verification(&state.pool, user.id).await?;
    let verification_documents = if let Some(item) = &verification {
        fetch_verification_documents(&state.pool, item.id).await?
    } else {
        Vec::new()
    };
    let liveness_completed = verification
        .as_ref()
        .map(|item| {
            matches!(
                item.status.as_str(),
                "submitted" | "pending" | "in_review" | "approved" | "verified"
            ) && verification_documents
                .iter()
                .any(|doc| doc.document_type == "selfie")
        })
        .unwrap_or(false);

    Ok(Json(AuthMeResponse {
        user: UserPublicView::from(refreshed_user),
        profile,
        role_profile,
        verification,
        verification_documents,
        liveness_completed,
        policy_metadata: PolicyMetadataView {
            terms_version: "1.0".to_string(),
            privacy_version: "1.0".to_string(),
            effective_at: Utc::now(),
            change_summary: "".to_string(),
            updated_at: Utc::now(),
        },
        policy_acceptance: PolicyAcceptanceView {
            terms_version_accepted: None,
            privacy_version_accepted: None,
            accepted_at: None,
            requires_reacceptance: false,
        },
    }))
}

pub async fn get_agent_notification_settings(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<crate::domain::users::AgentNotificationSettingsView>, AppError> {
    if user.role != UserRole::Agent {
        return Err(AppError::forbidden(
            "only agents can view notification settings",
        ));
    }

    let refreshed = state
        .user_repository
        .find_by_id(user.id)
        .await?
        .ok_or_else(|| AppError::not_found("user not found"))?;

    Ok(Json(crate::domain::users::AgentNotificationSettingsView {
        notifications_enabled: refreshed.notifications_enabled,
        operating_city: refreshed.operating_city,
        operating_state: refreshed.operating_state,
    }))
}

pub async fn update_agent_notification_settings(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<crate::domain::users::UpdateAgentNotificationSettingsInput>,
) -> Result<Json<crate::domain::users::AgentNotificationSettingsView>, AppError> {
    let settings = state
        .user_use_cases
        .update_agent_notification_settings(&user, payload)
        .await?;
    Ok(Json(settings))
}

pub async fn create_verification(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateVerificationInput>,
) -> Result<(StatusCode, Json<VerificationView>), AppError> {
    if !matches!(
        user.role,
        UserRole::Seeker | UserRole::Agent | UserRole::Landlord
    ) {
        return Err(AppError::forbidden(
            "only seeker, agent, and landlord accounts can submit verification",
        ));
    }

    let verification = sqlx::query_as::<_, VerificationView>(
        r#"
        INSERT INTO verifications (id, user_id, status, submitted_at, notes)
        VALUES ($1, $2, 'submitted', NOW(), $3)
        RETURNING id, user_id, status, submitted_at, reviewed_at, reviewed_by, rejection_reason,
                  notes, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user.id)
    .bind(payload.notes)
    .fetch_one(&state.pool)
    .await?;

    // Update user's verification_status to pending when verification is submitted
    if matches!(user.role, UserRole::Agent | UserRole::Landlord) {
        sqlx::query(
            r#"
            UPDATE users
            SET verification_status = 'pending',
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    }

    Ok((StatusCode::CREATED, Json(verification)))
}

pub async fn create_verification_document(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(verification_id): Path<Uuid>,
    Json(payload): Json<VerificationDocumentInput>,
) -> Result<(StatusCode, Json<VerificationDocumentView>), AppError> {
    let owns_verification = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM verifications WHERE id = $1 AND user_id = $2)",
    )
    .bind(verification_id)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    if !owns_verification {
        return Err(AppError::not_found("verification not found"));
    }

    let document = sqlx::query_as::<_, VerificationDocumentView>(
        r#"
        INSERT INTO verification_documents (id, verification_id, document_type, file_url, file_key, mime_type)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, verification_id, document_type, file_url, file_key, mime_type, status, created_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(verification_id)
    .bind(payload.document_type)
    .bind(payload.file_url)
    .bind(payload.file_key)
    .bind(payload.mime_type)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(document)))
}

pub async fn get_my_verification(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    if let Some(verification) = fetch_latest_verification(&state.pool, user.id).await? {
        let documents = fetch_verification_documents(&state.pool, verification.id).await?;
        return Ok(Json(json!({
            "verification": verification,
            "documents": documents
        })));
    }

    Ok(Json(json!({
        "verification": {
            "status": "not_started"
        },
        "documents": []
    })))
}

pub async fn list_agent_properties(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<PropertyListItem>>, AppError> {
    if user.role != UserRole::Agent {
        return Err(AppError::forbidden("only agents can view agent properties"));
    }
    let items = sqlx::query_as::<_, PropertyListItem>(
        r#"
        SELECT
            p.id,
            p.title,
            p.price,
            p.location,
            p.description,
            p.images,
            p.is_service_apartment,
            p.listing_type,
            p.status,
            p.self_managed,
            p.owner_id,
            p.agent_id,
            owner.full_name AS owner_name,
            agent.full_name AS agent_name,
            owner.phone AS owner_phone,
            agent.phone AS agent_phone,
            p.created_at,
            p.verified_at,
            COALESCE(view_stats.view_count, 0) AS view_count,
            COALESCE(offer_stats.offer_count, 0) AS offer_count,
            pending_change.id AS pending_price_request_id,
            pending_change.requested_price AS pending_requested_price,
            pending_change.created_at AS pending_price_requested_at,
            status_lock.available_at AS status_locked_until
        FROM properties p
        INNER JOIN users owner ON owner.id = p.owner_id
        LEFT JOIN users agent ON agent.id = p.agent_id
        LEFT JOIN (
            SELECT property_id, COUNT(*)::bigint AS view_count
            FROM property_views
            GROUP BY property_id
        ) view_stats ON view_stats.property_id = p.id
        LEFT JOIN (
            SELECT property_id, COUNT(*)::bigint AS offer_count
            FROM offers
            GROUP BY property_id
        ) offer_stats ON offer_stats.property_id = p.id
        LEFT JOIN property_change_requests pending_change
            ON pending_change.property_id = p.id
           AND pending_change.request_type = 'price_increase'
           AND pending_change.status = 'pending'
        LEFT JOIN LATERAL (
            SELECT prp.available_at
            FROM property_rental_periods prp
            WHERE prp.property_id = p.id AND prp.available_at IS NOT NULL
            ORDER BY prp.available_at DESC
            LIMIT 1
        ) status_lock ON TRUE
        WHERE p.agent_id = $1 OR p.owner_id = $1
        ORDER BY p.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn create_seeker_need(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreatePostInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden("only seekers can create needs"));
    }
    let id = state.post_use_cases.create_post(&user, payload).await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

pub async fn list_seeker_needs(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<PostListItem>>, AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden("only seekers can list needs"));
    }
    let items = sqlx::query_as::<_, PostListItem>(
        r#"
        SELECT
            p.id,
            p.author_id,
            u.full_name AS author_name,
            u.role::text AS author_role,
            p.location,
            p.request_title,
            p.area,
            p.city,
            p.state,
            p.property_type,
            p.bedrooms,
            p.min_budget,
            p.max_budget,
            p.pricing_preference,
            p.desired_features,
            p.target_agent_id,
            p.target_property_id,
            p.target_property_title,
            p.target_property_image_url,
            p.target_property_location,
            p.status,
            p.description,
            COALESCE(offer_stats.offer_count, 0) AS response_count,
            p.created_at
        FROM posts p
        INNER JOIN users u ON u.id = p.author_id
        LEFT JOIN (
            SELECT need_post_id, COUNT(*)::bigint AS offer_count
            FROM offers
            GROUP BY need_post_id
        ) offer_stats ON offer_stats.need_post_id = p.id
        WHERE p.author_id = $1
        ORDER BY p.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_agent_leads(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<AgentLeadView>>, AppError> {
    if user.role != UserRole::Agent {
        return Err(AppError::forbidden("only agents can view leads"));
    }
    let items = sqlx::query_as::<_, AgentLeadView>(
        r#"
        SELECT
            COALESCE(lm.id, apn.id) AS id,
            p.id AS need_post_id,
            lm.matched_property_id,
            p.target_property_id,
            p.target_property_title,
            p.target_property_image_url,
            p.target_property_location,
            COALESCE(lm.match_score, 0)::double precision AS match_score,
            COALESCE(lm.status, CASE WHEN apn.is_read THEN 'viewed' ELSE 'new' END) AS status,
            lm.sla_expires_at,
            COALESCE(lm.created_at, apn.created_at) AS created_at,
            COALESCE(lm.updated_at, apn.created_at) AS updated_at,
            p.request_title,
            p.location,
            p.property_type,
            NULL::text AS urgency
        FROM agent_post_notifications apn
        INNER JOIN posts p ON p.id = apn.post_id
        LEFT JOIN lead_matches lm ON lm.agent_user_id = apn.agent_id AND lm.need_post_id = apn.post_id
        WHERE apn.agent_id = $1
        ORDER BY COALESCE(lm.created_at, apn.created_at) DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn create_offer(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateOfferInput>,
) -> Result<(StatusCode, Json<OfferView>), AppError> {
    if !matches!(user.role, UserRole::Agent | UserRole::Landlord) {
        return Err(AppError::forbidden(
            "only agents and landlords can send offers",
        ));
    }
    if payload.message.trim().is_empty() {
        return Err(AppError::bad_request("message is required"));
    }
    let can_use_property = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM properties
            WHERE id = $1
              AND (owner_id = $2 OR agent_id = $2)
        )
        "#,
    )
    .bind(payload.property_id)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    if !can_use_property {
        return Err(AppError::forbidden(
            "you can only send offers with properties you own or manage",
        ));
    }

    let (seeker_user_id, request_title, property_title, agent_id, property_owner_id) =
        sqlx::query_as::<_, (Uuid, String, String, Option<Uuid>, Uuid)>(
            r#"
        SELECT p.author_id, p.request_title, property.title, property.agent_id, property.owner_id
        FROM posts p
        INNER JOIN properties property ON property.id = $2
        WHERE p.id = $1
        "#,
        )
        .bind(payload.need_post_id)
        .bind(payload.property_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("need post not found"))?;

    let offer = sqlx::query_as::<_, OfferView>(
        r#"
        INSERT INTO offers (
            id, need_post_id, provider_user_id, provider_role, property_id, lead_match_id,
            offer_price_amount, offer_price_currency, offer_price_period, move_in_date,
            custom_terms, message, priority_send
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, 'NGN'), COALESCE($9, 'year'), $10, $11, $12, COALESCE($13, FALSE))
        RETURNING id, need_post_id, provider_user_id, provider_role, property_id, lead_match_id,
                  offer_price_amount, offer_price_currency, offer_price_period, move_in_date,
                  custom_terms, message, priority_send, status, sent_at, viewed_at, accepted_at,
                  declined_at, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.need_post_id)
    .bind(user.id)
    .bind(user.role.as_str())
    .bind(payload.property_id)
    .bind(payload.lead_match_id)
    .bind(payload.offer_price_amount)
    .bind(payload.offer_price_currency)
    .bind(payload.offer_price_period)
    .bind(payload.move_in_date)
    .bind(payload.custom_terms)
    .bind(payload.message)
    .bind(payload.priority_send)
    .fetch_one(&state.pool)
    .await?;

    if user.role == UserRole::Agent {
        let updated = sqlx::query(
            r#"
            UPDATE lead_matches
            SET matched_property_id = $3,
                status = 'responded',
                updated_at = NOW()
            WHERE agent_user_id = $1
              AND need_post_id = $2
            "#,
        )
        .bind(user.id)
        .bind(payload.need_post_id)
        .bind(payload.property_id)
        .execute(&state.pool)
        .await?;

        if updated.rows_affected() == 0 {
            sqlx::query(
                r#"
                INSERT INTO lead_matches (
                    id, agent_user_id, need_post_id, matched_property_id, match_score, status, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, 100, 'responded', NOW(), NOW())
                "#,
            )
            .bind(payload.lead_match_id.unwrap_or_else(Uuid::new_v4))
            .bind(user.id)
            .bind(payload.need_post_id)
            .bind(payload.property_id)
            .execute(&state.pool)
            .await?;
        }
    }

    // Fetch seeker and agent/landlord details for email
    let seeker = sqlx::query_as::<_, (String, String)>("SELECT email, full_name FROM users WHERE id = $1")
        .bind(seeker_user_id)
        .fetch_optional(&state.pool)
        .await?;

    let property_contact = if agent_id.is_some() {
        sqlx::query_as::<_, (String, String)>("SELECT email, full_name FROM users WHERE id = $1")
            .bind(agent_id.unwrap())
            .fetch_optional(&state.pool)
            .await?
    } else {
        sqlx::query_as::<_, (String, String)>("SELECT email, full_name FROM users WHERE id = $1")
            .bind(property_owner_id)
            .fetch_optional(&state.pool)
            .await?
    };

    // Send notification to seeker
    insert_notification(
        &state.pool,
        seeker_user_id,
        "offer_received",
        "New offer received",
        &format!(
            "{} sent an offer for {} on your request \"{}\".",
            user.full_name, property_title, request_title
        ),
        Some("/seeker/offers"),
        json!({
            "offerId": offer.id,
            "propertyId": payload.property_id,
            "needPostId": payload.need_post_id,
            "providerUserId": user.id
        }),
    )
    .await?;

    // Send email to seeker
    if let Some((seeker_email, seeker_name)) = seeker {
        let header_image_url = header_asset_url(HEADER_NEW_MATCH);
        let offer_email = state.mail_service.offer_received_email(
            seeker_email,
            &seeker_name,
            &user.full_name,
            &property_title,
            &request_title,
            "https://verinest.ng/seeker/offers",
            &header_image_url,
        );
        let _ = state.mail_service.send(offer_email).await;
    }

    // Send notification to agent/landlord
    let contact_id = if agent_id.is_some() { agent_id.unwrap() } else { property_owner_id };
    insert_notification(
        &state.pool,
        contact_id,
        "offer_sent",
        "Offer sent to seeker",
        &format!(
            "Your offer for {} on \"{}\" has been sent.",
            property_title, request_title
        ),
        Some("/provider/inbox"),
        json!({
            "offerId": offer.id,
            "propertyId": payload.property_id,
            "needPostId": payload.need_post_id,
            "seekerId": seeker_user_id
        }),
    )
    .await?;

    // Send email to agent/landlord
    if let Some((contact_email, contact_name)) = property_contact {
        let header_image_url = header_asset_url(HEADER_LEAD_ALERT);
        let offer_email = state.mail_service.offer_received_email(
            contact_email,
            &contact_name,
            &user.full_name,
            &property_title,
            &request_title,
            "https://verinest.ng/provider/inbox",
            &header_image_url,
        );
        let _ = state.mail_service.send(offer_email).await;
    }

    Ok((StatusCode::CREATED, Json(offer)))
}

pub async fn update_seeker_offer(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOfferStatusInput>,
) -> Result<Json<OfferView>, AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden("only seekers can update offers"));
    }

    let normalized_status = payload.status.trim().to_lowercase();
    if normalized_status != "declined" {
        return Err(AppError::bad_request("invalid offer status"));
    }

    let offer = sqlx::query_as::<_, OfferView>(
        r#"
        UPDATE offers o
        SET status = 'declined',
            viewed_at = COALESCE(o.viewed_at, NOW()),
            declined_at = NOW(),
            updated_at = NOW()
        FROM posts need
        WHERE o.id = $1
          AND need.id = o.need_post_id
          AND need.author_id = $2
        RETURNING o.id, o.need_post_id, o.provider_user_id, o.provider_role, o.property_id, o.lead_match_id,
                  o.offer_price_amount, o.offer_price_currency, o.offer_price_period, o.move_in_date,
                  o.custom_terms, o.message, o.priority_send, o.status, o.sent_at, o.viewed_at,
                  o.accepted_at, o.declined_at, o.created_at, o.updated_at
        "#,
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("offer not found"))?;

    insert_notification(
        &state.pool,
        offer.provider_user_id,
        "offer_declined",
        "Offer declined",
        "A seeker declined your property offer.",
        Some("/provider/inbox"),
        json!({
            "offerId": offer.id,
            "propertyId": offer.property_id,
            "needPostId": offer.need_post_id
        }),
    )
    .await?;

    Ok(Json(offer))
}

pub async fn list_seeker_offers(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<OfferView>>, AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden("only seekers can view offers"));
    }
    let items = sqlx::query_as::<_, OfferView>(
        r#"
        SELECT
            o.id, o.need_post_id, o.provider_user_id, o.provider_role, o.property_id, o.lead_match_id,
            o.offer_price_amount, o.offer_price_currency, o.offer_price_period, o.move_in_date,
            o.custom_terms, o.message, o.priority_send, o.status, o.sent_at, o.viewed_at,
            o.accepted_at, o.declined_at, o.created_at, o.updated_at,
            property.title AS property_title,
            property.location AS property_location,
            property.images AS property_images,
            provider.full_name AS provider_name,
            provider.phone AS provider_phone,
            profiles.avatar_url AS provider_image_url
        FROM offers o
        INNER JOIN posts p ON p.id = o.need_post_id
        INNER JOIN properties property ON property.id = o.property_id
        INNER JOIN users provider ON provider.id = o.provider_user_id
        LEFT JOIN profiles ON profiles.user_id = o.provider_user_id
        WHERE p.author_id = $1
        ORDER BY o.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn create_saved_property(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<SavePropertyInput>,
) -> Result<(StatusCode, Json<SavedPropertyView>), AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden("only seekers can save properties"));
    }
    let item = sqlx::query_as::<_, SavedPropertyView>(
        r#"
        INSERT INTO saved_properties (id, user_id, property_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, property_id) DO UPDATE SET property_id = EXCLUDED.property_id
        RETURNING id, user_id, property_id, created_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user.id)
    .bind(payload.property_id)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn delete_saved_property(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(property_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden(
            "only seekers can remove saved properties",
        ));
    }
    sqlx::query("DELETE FROM saved_properties WHERE user_id = $1 AND property_id = $2")
        .bind(user.id)
        .bind(property_id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_saved_properties(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<Value>>, AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden(
            "only seekers can view saved properties",
        ));
    }
    let items = sqlx::query(
        r#"
        SELECT
            sp.id,
            sp.user_id,
            sp.property_id,
            sp.created_at,
            p.title,
            p.price,
            p.location,
            p.images,
            owner.full_name AS owner_name,
            agent.full_name AS agent_name,
            COALESCE(view_stats.view_count, 0) AS view_count
        FROM saved_properties sp
        INNER JOIN properties p ON p.id = sp.property_id
        INNER JOIN users owner ON owner.id = p.owner_id
        LEFT JOIN users agent ON agent.id = p.agent_id
        LEFT JOIN (
            SELECT property_id, COUNT(*)::bigint AS view_count
            FROM property_views
            GROUP BY property_id
        ) view_stats ON view_stats.property_id = p.id
        WHERE sp.user_id = $1
        ORDER BY sp.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(
        items
            .into_iter()
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "userId": row.get::<Uuid, _>("user_id"),
                    "propertyId": row.get::<Uuid, _>("property_id"),
                    "createdAt": row.get::<DateTime<Utc>, _>("created_at"),
                    "title": row.get::<String, _>("title"),
                    "price": row.get::<i64, _>("price"),
                    "location": row.get::<String, _>("location"),
                    "images": row.get::<Vec<String>, _>("images"),
                    "ownerName": row.get::<String, _>("owner_name"),
                    "agentName": row.try_get::<Option<String>, _>("agent_name").ok().flatten(),
                    "viewCount": row.get::<i64, _>("view_count")
                })
            })
            .collect(),
    ))
}

pub async fn create_review(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<ApiCreateReviewInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    if payload.rating < 1 || payload.rating > 5 {
        return Err(AppError::bad_request("rating must be between 1 and 5"));
    }
    if payload.comment.trim().is_empty() {
        return Err(AppError::bad_request("comment is required"));
    }
    if payload.reviewee_id == user.id {
        return Err(AppError::bad_request("you cannot review yourself"));
    }

    let reviewee_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
            .bind(payload.reviewee_id)
            .fetch_one(&state.pool)
            .await?;
    if !reviewee_exists {
        return Err(AppError::not_found("reviewee not found"));
    }

    if let Some(property_id) = payload.property_id {
        let property_owner = sqlx::query_as::<_, (Uuid, Option<Uuid>)>(
            "SELECT owner_id, agent_id FROM properties WHERE id = $1",
        )
        .bind(property_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("property not found"))?;
        let valid_target = payload.reviewee_id == property_owner.0
            || Some(payload.reviewee_id) == property_owner.1;
        if !valid_target {
            return Err(AppError::bad_request(
                "reviewee does not match this property",
            ));
        }
    }

    let has_context = if let Some(property_id) = payload.property_id {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM bookings
                WHERE seeker_user_id = $1
                  AND provider_user_id = $2
                  AND property_id = $3
                UNION
                SELECT 1
                FROM offers o
                INNER JOIN posts p ON p.id = o.need_post_id
                WHERE p.author_id = $1
                  AND o.provider_user_id = $2
                  AND o.property_id = $3
            )
            "#,
        )
        .bind(user.id)
        .bind(payload.reviewee_id)
        .bind(property_id)
        .fetch_one(&state.pool)
        .await?
    } else {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM bookings
                WHERE seeker_user_id = $1
                  AND provider_user_id = $2
                UNION
                SELECT 1
                FROM offers o
                INNER JOIN posts p ON p.id = o.need_post_id
                WHERE p.author_id = $1
                  AND o.provider_user_id = $2
            )
            "#,
        )
        .bind(user.id)
        .bind(payload.reviewee_id)
        .fetch_one(&state.pool)
        .await?
    };

    if !has_context {
        return Err(AppError::forbidden(
            "reviews require an offer or booking history with this agent or property",
        ));
    }

    let review = sqlx::query_scalar::<_, Value>(
        r#"
        WITH inserted AS (
            INSERT INTO reviews (id, reviewer_id, reviewee_id, property_id, response_id, rating, comment)
            VALUES ($1, $2, $3, $4, NULL, $5, $6)
            RETURNING id, reviewer_id, reviewee_id, property_id, response_id, rating, comment, created_at
        )
        SELECT to_jsonb(inserted)
        FROM inserted
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user.id)
    .bind(payload.reviewee_id)
    .bind(payload.property_id)
    .bind(payload.rating)
    .bind(payload.comment.trim())
    .fetch_one(&state.pool)
    .await?;

    if let Some(property_id) = payload.property_id {
        let low_review_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)::bigint FROM reviews WHERE property_id = $1 AND rating <= 2",
        )
        .bind(property_id)
        .fetch_one(&state.pool)
        .await?;
        if low_review_count >= 3 {
            sqlx::query(
                "UPDATE properties SET status = 'suspended', updated_at = NOW() WHERE id = $1",
            )
            .bind(property_id)
            .execute(&state.pool)
            .await?;
        }
    }

    if let Err(error) = insert_notification(
        &state.pool,
        payload.reviewee_id,
        "review_received",
        "New review received",
        "A user left a review on your profile or property.",
        Some("/provider/settings"),
        json!({
            "revieweeId": payload.reviewee_id,
            "propertyId": payload.property_id,
            "rating": payload.rating
        }),
    )
    .await
    {
        tracing::error!("failed to create review notification: {error:?}");
    }

    Ok((StatusCode::CREATED, Json(review)))
}

pub async fn list_property_reviews(
    State(state): State<AppState>,
    Path(property_id): Path<Uuid>,
) -> Result<Json<Vec<PropertyReviewView>>, AppError> {
    let items = sqlx::query_as::<_, PropertyReviewView>(
        r#"
        SELECT
            r.id,
            r.reviewer_id,
            reviewer.full_name AS reviewer_name,
            r.reviewee_id,
            reviewee.full_name AS reviewee_name,
            r.property_id,
            r.rating,
            r.comment,
            r.created_at
        FROM reviews r
        INNER JOIN users reviewer ON reviewer.id = r.reviewer_id
        INNER JOIN users reviewee ON reviewee.id = r.reviewee_id
        WHERE r.property_id = $1
        ORDER BY r.created_at DESC
        "#,
    )
    .bind(property_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn create_booking(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateBookingInput>,
) -> Result<(StatusCode, Json<BookingView>), AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden("only seekers can create bookings"));
    }

    if !matches!(
        payload.booking_type.as_str(),
        "viewing" | "hold" | "move_in" | "shortlet"
    ) {
        return Err(AppError::bad_request("invalid booking_type"));
    }

    let offer_context = sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"
        SELECT o.provider_user_id, o.property_id
        FROM offers o
        INNER JOIN posts p ON p.id = o.need_post_id
        WHERE o.id = $1
          AND p.author_id = $2
          AND o.status NOT IN ('declined', 'withdrawn', 'expired')
        "#,
    )
    .bind(payload.offer_id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("offer not found"))?;
    let (provider_user_id, offer_property_id) = offer_context;
    if offer_property_id != payload.property_id {
        return Err(AppError::bad_request(
            "booking property does not match offer",
        ));
    }

    let booking_type = payload.booking_type.clone();

    let booking = sqlx::query_as::<_, BookingView>(
        r#"
        INSERT INTO bookings (id, offer_id, property_id, seeker_user_id, provider_user_id, booking_type, scheduled_for, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, offer_id, property_id, unit_id, seeker_user_id, provider_user_id,
                  booking_type, scheduled_for, status, notes, created_at, updated_at,
                  confirmed_at,
                  NULL::text AS property_title,
                  NULL::text AS property_location,
                  NULL::text AS provider_name,
                  NULL::text AS seeker_name,
                  NULL::text AS provider_phone,
                  NULL::text AS provider_avatar_url,
                  NULL::text AS property_listing_type,
                  seeker_outcome, seeker_outcome_note, seeker_outcome_confirmed_at,
                  provider_outcome, provider_outcome_note, provider_outcome_confirmed_at,
                  outcome_resolution, outcome_follow_up_required, listing_outcome_applied
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.offer_id)
    .bind(payload.property_id)
    .bind(user.id)
    .bind(provider_user_id)
    .bind(payload.booking_type)
    .bind(payload.scheduled_for)
    .bind(payload.notes)
    .fetch_one(&state.pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE offers
        SET status = 'shortlisted',
            viewed_at = COALESCE(viewed_at, NOW()),
            updated_at = NOW()
        WHERE id = $1
          AND status IN ('sent', 'viewed', 'shortlisted', 'negotiated')
        "#,
    )
    .bind(payload.offer_id)
    .execute(&state.pool)
    .await?;

    let property_title =
        sqlx::query_scalar::<_, String>("SELECT title FROM properties WHERE id = $1")
            .bind(payload.property_id)
            .fetch_optional(&state.pool)
            .await?
            .unwrap_or_else(|| "property".to_string());

    insert_notification(
        &state.pool,
        provider_user_id,
        "booking_requested",
        "New site visit requested",
        &format!(
            "A seeker requested a {} for {}.",
            booking_type.replace('_', " "),
            property_title
        ),
        Some("/provider/calendar"),
        json!({
            "bookingId": booking.id,
            "offerId": payload.offer_id,
            "propertyId": payload.property_id,
            "scheduledFor": booking.scheduled_for
        }),
    )
    .await?;

    insert_notification(
        &state.pool,
        user.id,
        "booking_created",
        "Visit scheduled",
        &format!(
            "Your {} for {} has been scheduled.",
            booking_type.replace('_', " "),
            property_title
        ),
        Some("/seeker/bookings"),
        json!({
            "bookingId": booking.id,
            "offerId": payload.offer_id,
            "propertyId": payload.property_id,
            "scheduledFor": booking.scheduled_for
        }),
    )
    .await?;

    Ok((StatusCode::CREATED, Json(booking)))
}

pub async fn update_booking(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBookingInput>,
) -> Result<Json<BookingView>, AppError> {
    if payload.scheduled_for.is_none() && payload.notes.is_none() && payload.status.is_none() {
        return Err(AppError::bad_request(
            "at least one booking field must be provided",
        ));
    }
    if let Some(status) = payload.status.as_deref() {
        if !matches!(
            status,
            "pending" | "confirmed" | "completed" | "cancelled" | "no_show"
        ) {
            return Err(AppError::bad_request("invalid booking status"));
        }
    }

    let existing = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM bookings
            WHERE id = $1
              AND (seeker_user_id = $2 OR provider_user_id = $2)
        )
        "#,
    )
    .bind(id)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    if !existing {
        return Err(AppError::not_found("booking not found"));
    }

    let booking = sqlx::query_as::<_, BookingView>(
        r#"
        UPDATE bookings b
        SET scheduled_for = COALESCE($2, b.scheduled_for),
            notes = CASE
                WHEN $3 IS NOT NULL AND b.notes IS NOT NULL AND LENGTH(TRIM(b.notes)) > 0 THEN b.notes || E'\n\n' || TRIM($3)
                WHEN $3 IS NOT NULL THEN TRIM($3)
                ELSE b.notes
            END,
            status = COALESCE($4, b.status),
            updated_at = NOW()
        WHERE b.id = $1
          AND (b.seeker_user_id = $5 OR b.provider_user_id = $5)
        RETURNING b.id, b.offer_id, b.property_id, b.unit_id, b.seeker_user_id, b.provider_user_id,
                  b.booking_type, b.scheduled_for, b.status, b.notes, b.created_at, b.updated_at,
                  b.confirmed_at,
                  NULL::text AS property_title,
                  NULL::text AS property_location,
                  NULL::text AS provider_name,
                  NULL::text AS seeker_name,
                  NULL::text AS provider_phone,
                  NULL::text AS provider_avatar_url,
                  NULL::text AS property_listing_type,
                  b.seeker_outcome, b.seeker_outcome_note, b.seeker_outcome_confirmed_at,
                  b.provider_outcome, b.provider_outcome_note, b.provider_outcome_confirmed_at,
                  b.outcome_resolution, b.outcome_follow_up_required, b.listing_outcome_applied
        "#,
    )
    .bind(id)
    .bind(payload.scheduled_for)
    .bind(payload.notes.as_deref())
    .bind(payload.status.as_deref())
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("booking not found"))?;

    let counterparty_user_id = if booking.seeker_user_id == user.id {
        booking.provider_user_id
    } else {
        booking.seeker_user_id
    };
    let action_url = if booking.provider_user_id == counterparty_user_id {
        "/provider/calendar"
    } else {
        "/seeker/bookings"
    };
    let actor_label = if user.role == UserRole::Agent {
        "Agent"
    } else {
        "Seeker"
    };

    insert_notification(
        &state.pool,
        counterparty_user_id,
        "booking_updated",
        "Booking updated",
        &format!("{actor_label} updated the booking schedule or meetup notes."),
        Some(action_url),
        json!({
            "bookingId": booking.id,
            "propertyId": booking.property_id,
            "scheduledFor": booking.scheduled_for,
            "status": booking.status
        }),
    )
    .await?;

    Ok(Json(booking))
}

pub async fn confirm_booking_schedule(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<BookingView>, AppError> {
    if user.role != UserRole::Agent {
        return Err(AppError::forbidden("only agents can confirm bookings"));
    }

    let booking = sqlx::query_as::<_, BookingView>(
        r#"
        UPDATE bookings b
        SET confirmed_at = NOW(),
            updated_at = NOW()
        WHERE b.id = $1
          AND b.provider_user_id = $2
          AND b.confirmed_at IS NULL
        RETURNING b.id, b.offer_id, b.property_id, b.unit_id, b.seeker_user_id, b.provider_user_id,
                  b.booking_type, b.scheduled_for, b.status, b.notes, b.created_at, b.updated_at,
                  b.confirmed_at,
                  NULL::text AS property_title,
                  NULL::text AS property_location,
                  NULL::text AS provider_name,
                  NULL::text AS seeker_name,
                  NULL::text AS provider_phone,
                  NULL::text AS provider_avatar_url,
                  NULL::text AS property_listing_type,
                  b.seeker_outcome, b.seeker_outcome_note, b.seeker_outcome_confirmed_at,
                  b.provider_outcome, b.provider_outcome_note, b.provider_outcome_confirmed_at,
                  b.outcome_resolution, b.outcome_follow_up_required, b.listing_outcome_applied
        "#,
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("booking not found or already confirmed"))?;

    // Notify seeker that agent confirmed
    insert_notification(
        &state.pool,
        booking.seeker_user_id,
        "booking_confirmed",
        "Agent confirmed your visit",
        &format!("Agent confirmed your scheduled visit. You can now see their contact details."),
        Some("/seeker/bookings"),
        json!({
            "bookingId": booking.id,
            "propertyId": booking.property_id,
            "scheduledFor": booking.scheduled_for
        }),
    )
    .await?;

    Ok(Json(booking))
}

async fn notify_admin_booking_outcome_conflict(
    pool: &PgPool,
    booking_id: Uuid,
    property_id: Uuid,
    seeker_user_id: Uuid,
    provider_user_id: Uuid,
    seeker_outcome: &str,
    provider_outcome: &str,
) -> Result<(), AppError> {
    let admin_ids = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE role = 'admin'")
        .fetch_all(pool)
        .await?;

    for admin_id in admin_ids {
        insert_notification(
            pool,
            admin_id,
            "booking_outcome_conflict",
            "Booking outcome conflict",
            "A seeker and provider submitted conflicting booking outcomes that need follow-up.",
            Some("/admin/reports"),
            json!({
                "bookingId": booking_id,
                "propertyId": property_id,
                "seekerUserId": seeker_user_id,
                "providerUserId": provider_user_id,
                "seekerOutcome": seeker_outcome,
                "providerOutcome": provider_outcome
            }),
        )
        .await?;
    }

    Ok(())
}

async fn resolve_booking_outcome_if_ready(
    pool: &PgPool,
    booking_id: Uuid,
) -> Result<BookingView, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            b.id,
            b.offer_id,
            b.property_id,
            b.unit_id,
            b.seeker_user_id,
            b.provider_user_id,
            b.booking_type,
            b.scheduled_for,
            b.status,
            b.notes,
            b.created_at,
            b.updated_at,
            b.seeker_outcome,
            b.seeker_outcome_note,
            b.seeker_outcome_confirmed_at,
            b.provider_outcome,
            b.provider_outcome_note,
            b.provider_outcome_confirmed_at,
            b.outcome_resolution,
            b.outcome_follow_up_required,
            b.listing_outcome_applied,
            p.title AS property_title,
            p.location AS property_location,
            p.listing_type AS property_listing_type,
            provider.full_name AS provider_name,
            seeker.full_name AS seeker_name,
            provider.phone AS provider_phone,
            provider_profile.avatar_url AS provider_avatar_url
        FROM bookings b
        INNER JOIN properties p ON p.id = b.property_id
        INNER JOIN users provider ON provider.id = b.provider_user_id
        INNER JOIN users seeker ON seeker.id = b.seeker_user_id
        LEFT JOIN profiles provider_profile ON provider_profile.user_id = provider.id
        WHERE b.id = $1
        "#,
    )
    .bind(booking_id)
    .fetch_one(pool)
    .await?;

    let seeker_outcome = row.try_get::<Option<String>, _>("seeker_outcome")?;
    let provider_outcome = row.try_get::<Option<String>, _>("provider_outcome")?;
    let property_id = row.get::<Uuid, _>("property_id");
    let unit_id = row.try_get::<Option<Uuid>, _>("unit_id")?;
    let listing_type = row
        .try_get::<Option<String>, _>("property_listing_type")?
        .unwrap_or_else(|| "rent".to_string());
    let seeker_user_id = row.get::<Uuid, _>("seeker_user_id");
    let provider_user_id = row.get::<Uuid, _>("provider_user_id");

    if let (Some(seeker_outcome), Some(provider_outcome)) =
        (seeker_outcome.clone(), provider_outcome.clone())
    {
        if seeker_outcome == provider_outcome {
            let resolution = if seeker_outcome == "completed" {
                "aligned_completed"
            } else {
                "aligned_not_completed"
            };
            let mut listing_outcome_applied = false;

            if seeker_outcome == "completed" {
                match listing_type.as_str() {
                    "sale" => {
                        sqlx::query(
                            "UPDATE properties SET status = 'sold_out', updated_at = NOW() WHERE id = $1",
                        )
                        .bind(property_id)
                        .execute(pool)
                        .await?;
                    }
                    "shortlet" => {
                        sqlx::query(
                            "UPDATE properties SET status = 'in_use', updated_at = NOW() WHERE id = $1",
                        )
                        .bind(property_id)
                        .execute(pool)
                        .await?;

                        if let Some(unit_id) = unit_id {
                            sqlx::query(
                                "UPDATE units SET occupancy_status = 'occupied', listing_status = 'paused', updated_at = NOW() WHERE id = $1",
                            )
                            .bind(unit_id)
                            .execute(pool)
                            .await?;
                        }
                    }
                    _ => {
                        sqlx::query(
                            "UPDATE properties SET status = 'rented_out', updated_at = NOW() WHERE id = $1",
                        )
                        .bind(property_id)
                        .execute(pool)
                        .await?;

                        if let Some(unit_id) = unit_id {
                            sqlx::query(
                                "UPDATE units SET occupancy_status = 'occupied', listing_status = 'paused', updated_at = NOW() WHERE id = $1",
                            )
                            .bind(unit_id)
                            .execute(pool)
                            .await?;
                        }
                    }
                }
                listing_outcome_applied = true;
            }

            sqlx::query(
                r#"
                UPDATE bookings
                SET outcome_resolution = $2,
                    outcome_follow_up_required = FALSE,
                    listing_outcome_applied = $3,
                    status = 'completed',
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(booking_id)
            .bind(resolution)
            .bind(listing_outcome_applied)
            .execute(pool)
            .await?;

            insert_notification(
                pool,
                seeker_user_id,
                "booking_outcome_resolved",
                "Booking outcome confirmed",
                "Both sides confirmed the outcome of this booking.",
                Some("/seeker/bookings"),
                json!({
                    "bookingId": booking_id,
                    "propertyId": property_id,
                    "resolution": resolution,
                    "listingOutcomeApplied": listing_outcome_applied
                }),
            )
            .await?;

            insert_notification(
                pool,
                provider_user_id,
                "booking_outcome_resolved",
                "Booking outcome confirmed",
                "Both sides confirmed the outcome of this booking.",
                Some("/provider/calendar"),
                json!({
                    "bookingId": booking_id,
                    "propertyId": property_id,
                    "resolution": resolution,
                    "listingOutcomeApplied": listing_outcome_applied
                }),
            )
            .await?;
        } else {
            sqlx::query(
                r#"
                UPDATE bookings
                SET outcome_resolution = 'conflict',
                    outcome_follow_up_required = TRUE,
                    listing_outcome_applied = FALSE,
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(booking_id)
            .execute(pool)
            .await?;

            notify_admin_booking_outcome_conflict(
                pool,
                booking_id,
                property_id,
                seeker_user_id,
                provider_user_id,
                &seeker_outcome,
                &provider_outcome,
            )
            .await?;

            insert_notification(
                pool,
                seeker_user_id,
                "booking_outcome_conflict",
                "Booking outcome needs follow-up",
                "Your booking outcome does not match the provider's confirmation. Support or admin review may follow.",
                Some("/seeker/bookings"),
                json!({
                    "bookingId": booking_id,
                    "propertyId": property_id
                }),
            )
            .await?;

            insert_notification(
                pool,
                provider_user_id,
                "booking_outcome_conflict",
                "Booking outcome needs follow-up",
                "Your booking outcome does not match the seeker's confirmation. Support or admin review may follow.",
                Some("/provider/calendar"),
                json!({
                    "bookingId": booking_id,
                    "propertyId": property_id
                }),
            )
            .await?;
        }
    }

    let booking = sqlx::query_as::<_, BookingView>(
        r#"
        SELECT
            b.id, b.offer_id, b.property_id, b.unit_id, b.seeker_user_id, b.provider_user_id,
            b.booking_type, b.scheduled_for, b.status, b.notes, b.created_at, b.updated_at,
            b.confirmed_at,
            p.title AS property_title,
            p.location AS property_location,
            provider.full_name AS provider_name,
            seeker.full_name AS seeker_name,
            provider.phone AS provider_phone,
            provider_profile.avatar_url AS provider_avatar_url,
            p.listing_type AS property_listing_type,
            b.seeker_outcome, b.seeker_outcome_note, b.seeker_outcome_confirmed_at,
            b.provider_outcome, b.provider_outcome_note, b.provider_outcome_confirmed_at,
            b.outcome_resolution, b.outcome_follow_up_required, b.listing_outcome_applied
        FROM bookings b
        INNER JOIN properties p ON p.id = b.property_id
        INNER JOIN users provider ON provider.id = b.provider_user_id
        INNER JOIN users seeker ON seeker.id = b.seeker_user_id
        LEFT JOIN profiles provider_profile ON provider_profile.user_id = provider.id
        WHERE b.id = $1
        "#,
    )
    .bind(booking_id)
    .fetch_one(pool)
    .await?;

    Ok(booking)
}

pub async fn confirm_booking_seeker_outcome(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<ConfirmBookingOutcomeInput>,
) -> Result<Json<BookingView>, AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden("only seekers can confirm booking outcomes"));
    }

    let outcome = payload.outcome.trim().to_lowercase();
    if !matches!(outcome.as_str(), "completed" | "not_completed") {
        return Err(AppError::bad_request("invalid booking outcome"));
    }

    let booking_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM bookings WHERE id = $1 AND seeker_user_id = $2)",
    )
    .bind(id)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    if !booking_exists {
        return Err(AppError::not_found("booking not found"));
    }

    let row = sqlx::query(
        r#"
        UPDATE bookings
        SET seeker_outcome = $2,
            seeker_outcome_note = $3,
            seeker_outcome_confirmed_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
        RETURNING provider_user_id, property_id, provider_outcome
        "#,
    )
    .bind(id)
    .bind(&outcome)
    .bind(payload.note.as_deref())
    .fetch_one(&state.pool)
    .await?;

    let provider_user_id = row.get::<Uuid, _>("provider_user_id");
    let property_id = row.get::<Uuid, _>("property_id");
    let provider_outcome = row.try_get::<Option<String>, _>("provider_outcome")?;

    if provider_outcome.is_none() {
        insert_notification(
            &state.pool,
            provider_user_id,
            "booking_outcome_confirmation_requested",
            "Seeker confirmed booking outcome",
            "The seeker has confirmed the outcome of this booking. Please confirm whether the visit completed successfully.",
            Some("/provider/calendar"),
            json!({
                "bookingId": id,
                "propertyId": property_id,
                "seekerOutcome": outcome
            }),
        )
        .await?;
    }

    Ok(Json(resolve_booking_outcome_if_ready(&state.pool, id).await?))
}

pub async fn confirm_booking_provider_outcome(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<ConfirmBookingOutcomeInput>,
) -> Result<Json<BookingView>, AppError> {
    if !matches!(user.role, UserRole::Agent | UserRole::Landlord) {
        return Err(AppError::forbidden("only providers can confirm booking outcomes"));
    }

    let outcome = payload.outcome.trim().to_lowercase();
    if !matches!(outcome.as_str(), "completed" | "not_completed") {
        return Err(AppError::bad_request("invalid booking outcome"));
    }

    let booking_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM bookings WHERE id = $1 AND provider_user_id = $2)",
    )
    .bind(id)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    if !booking_exists {
        return Err(AppError::not_found("booking not found"));
    }

    let row = sqlx::query(
        r#"
        UPDATE bookings
        SET provider_outcome = $2,
            provider_outcome_note = $3,
            provider_outcome_confirmed_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
        RETURNING seeker_user_id, property_id, seeker_outcome
        "#,
    )
    .bind(id)
    .bind(&outcome)
    .bind(payload.note.as_deref())
    .fetch_one(&state.pool)
    .await?;

    let seeker_user_id = row.get::<Uuid, _>("seeker_user_id");
    let property_id = row.get::<Uuid, _>("property_id");
    let seeker_outcome = row.try_get::<Option<String>, _>("seeker_outcome")?;

    if seeker_outcome.is_none() {
        insert_notification(
            &state.pool,
            seeker_user_id,
            "booking_outcome_confirmation_requested",
            "Provider confirmed booking outcome",
            "The provider has confirmed the outcome of this booking. Please confirm whether the visit completed successfully.",
            Some("/seeker/bookings"),
            json!({
                "bookingId": id,
                "propertyId": property_id,
                "providerOutcome": outcome
            }),
        )
        .await?;
    }

    Ok(Json(resolve_booking_outcome_if_ready(&state.pool, id).await?))
}

pub async fn create_booking_dispute(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateBookingDisputeInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    if !matches!(user.role, UserRole::Seeker | UserRole::Agent | UserRole::Landlord) {
        return Err(AppError::forbidden("only seekers and providers can raise booking disputes"));
    }

    let dispute_type = payload.dispute_type.trim().to_lowercase();
    if !matches!(
        dispute_type.as_str(),
        "fraud" | "quality" | "cancellation" | "payment" | "impersonation" | "listing_misrepresentation"
    ) {
        return Err(AppError::bad_request("invalid dispute type"));
    }

    let priority = payload
        .priority
        .as_deref()
        .unwrap_or("medium")
        .trim()
        .to_lowercase();
    if !matches!(priority.as_str(), "low" | "medium" | "high" | "critical") {
        return Err(AppError::bad_request("invalid dispute priority"));
    }

    let title = payload.title.trim();
    let description = payload.description.trim();
    if title.is_empty() {
        return Err(AppError::bad_request("title is required"));
    }
    if description.is_empty() {
        return Err(AppError::bad_request("description is required"));
    }

    let booking = sqlx::query(
        r#"
        SELECT
            b.id,
            b.offer_id,
            b.property_id,
            b.seeker_user_id,
            b.provider_user_id,
            b.status,
            b.scheduled_for,
            p.title AS property_title
        FROM bookings b
        INNER JOIN properties p ON p.id = b.property_id
        WHERE b.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("booking not found"))?;

    let seeker_user_id = booking.get::<Uuid, _>("seeker_user_id");
    let provider_user_id = booking.get::<Uuid, _>("provider_user_id");
    let booking_status = booking.get::<String, _>("status");
    let scheduled_for = booking.get::<DateTime<Utc>, _>("scheduled_for");
    let property_id = booking.get::<Uuid, _>("property_id");
    let offer_id = booking.try_get::<Option<Uuid>, _>("offer_id")?;
    let property_title = booking.get::<String, _>("property_title");

    if user.id != seeker_user_id && user.id != provider_user_id {
        return Err(AppError::forbidden("you are not part of this booking"));
    }

    if booking_status != "confirmed" {
        return Err(AppError::bad_request("disputes can only be raised for confirmed visits"));
    }

    if scheduled_for + chrono::Duration::hours(1) > Utc::now() {
        return Err(AppError::bad_request(
            "disputes can only be raised one hour after the scheduled visit time",
        ));
    }

    let subject_user_id = if user.id == seeker_user_id {
        provider_user_id
    } else {
        seeker_user_id
    };

    let dispute_id = Uuid::new_v4();
    let reference = format!("DSP-{}", &dispute_id.to_string()[..8].to_uppercase());

    sqlx::query(
        r#"
        INSERT INTO disputes (
            id, reference, reporter_user_id, subject_user_id, property_id, offer_id, booking_id,
            type, priority, status, title, description
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'open', $10, $11)
        "#,
    )
    .bind(dispute_id)
    .bind(&reference)
    .bind(user.id)
    .bind(subject_user_id)
    .bind(property_id)
    .bind(offer_id)
    .bind(id)
    .bind(&dispute_type)
    .bind(&priority)
    .bind(title)
    .bind(description)
    .execute(&state.pool)
    .await?;

    insert_notification(
        &state.pool,
        subject_user_id,
        "booking_dispute_opened",
        "A booking dispute was raised",
        "A dispute has been opened on a confirmed visit involving you. Support or admin review may follow.",
        Some(if user.id == seeker_user_id {
            "/provider/calendar"
        } else {
            "/seeker/bookings"
        }),
        json!({
            "disputeId": dispute_id,
            "bookingId": id,
            "propertyId": property_id,
            "propertyTitle": property_title,
            "type": dispute_type,
            "priority": priority,
        }),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": dispute_id,
            "reference": reference,
            "bookingId": id,
        })),
    ))
}

pub async fn list_seeker_bookings(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<BookingView>>, AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden("only seekers can view bookings"));
    }
    let items = sqlx::query_as::<_, BookingView>(
        r#"
        SELECT
            b.id, b.offer_id, b.property_id, b.unit_id, b.seeker_user_id, b.provider_user_id,
            b.booking_type, b.scheduled_for, b.status, b.notes, b.created_at, b.updated_at,
            p.title AS property_title,
            p.location AS property_location,
            provider.full_name AS provider_name,
            seeker.full_name AS seeker_name,
            provider.phone AS provider_phone,
            provider_profile.avatar_url AS provider_avatar_url,
            p.listing_type AS property_listing_type,
            b.seeker_outcome, b.seeker_outcome_note, b.seeker_outcome_confirmed_at,
            b.provider_outcome, b.provider_outcome_note, b.provider_outcome_confirmed_at,
            b.outcome_resolution, b.outcome_follow_up_required, b.listing_outcome_applied
        FROM bookings b
        INNER JOIN properties p ON p.id = b.property_id
        INNER JOIN users provider ON provider.id = b.provider_user_id
        INNER JOIN users seeker ON seeker.id = b.seeker_user_id
        LEFT JOIN profiles provider_profile ON provider_profile.user_id = provider.id
        WHERE b.seeker_user_id = $1
        ORDER BY b.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_agent_bookings(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<BookingView>>, AppError> {
    if user.role != UserRole::Agent {
        return Err(AppError::forbidden("only agents can view agent bookings"));
    }
    let items = sqlx::query_as::<_, BookingView>(
        r#"
        SELECT
            b.id, b.offer_id, b.property_id, b.unit_id, b.seeker_user_id, b.provider_user_id,
            b.booking_type, b.scheduled_for, b.status, b.notes, b.created_at, b.updated_at,
            p.title AS property_title,
            p.location AS property_location,
            provider.full_name AS provider_name,
            seeker.full_name AS seeker_name,
            provider.phone AS provider_phone,
            provider_profile.avatar_url AS provider_avatar_url,
            p.listing_type AS property_listing_type,
            b.seeker_outcome, b.seeker_outcome_note, b.seeker_outcome_confirmed_at,
            b.provider_outcome, b.provider_outcome_note, b.provider_outcome_confirmed_at,
            b.outcome_resolution, b.outcome_follow_up_required, b.listing_outcome_applied
        FROM bookings b
        INNER JOIN properties p ON p.id = b.property_id
        INNER JOIN users provider ON provider.id = b.provider_user_id
        INNER JOIN users seeker ON seeker.id = b.seeker_user_id
        LEFT JOIN profiles provider_profile ON provider_profile.user_id = provider.id
        WHERE b.provider_user_id = $1
        ORDER BY b.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_landlord_properties(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<PropertyListItem>>, AppError> {
    if user.role != UserRole::Landlord {
        return Err(AppError::forbidden(
            "only landlords can view landlord properties",
        ));
    }
    let items = sqlx::query_as::<_, PropertyListItem>(
        r#"
        SELECT
            p.id,
            p.title,
            p.price,
            p.location,
            p.description,
            p.images,
            p.is_service_apartment,
            p.listing_type,
            p.status,
            p.self_managed,
            p.owner_id,
            p.agent_id,
            owner.full_name AS owner_name,
            agent.full_name AS agent_name,
            p.created_at,
            p.verified_at
        FROM properties p
        INNER JOIN users owner ON owner.id = p.owner_id
        LEFT JOIN users agent ON agent.id = p.agent_id
        WHERE p.owner_id = $1
        ORDER BY p.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_landlord_units(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<UnitView>>, AppError> {
    ensure_landlord(&user)?;
    let items = sqlx::query_as::<_, UnitView>(
        r#"
        SELECT
            u.id, u.property_id, u.unit_code, u.name, u.unit_type, u.bedrooms_label,
            u.rent_amount, u.rent_currency, u.rent_period, u.occupancy_status,
            u.listing_status, u.tenant_user_id, u.lease_id, u.created_at, u.updated_at
        FROM units u
        INNER JOIN properties p ON p.id = u.property_id
        WHERE p.owner_id = $1
        ORDER BY u.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_landlord_collections(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<CollectionView>>, AppError> {
    ensure_landlord(&user)?;
    let items = sqlx::query_as::<_, CollectionView>(
        r#"
        SELECT
            rc.id, rc.lease_id, rc.unit_id, rc.tenant_user_id, rc.due_date, rc.amount,
            rc.currency, rc.status, rc.paid_amount, rc.created_at, rc.updated_at
        FROM rent_charges rc
        INNER JOIN leases l ON l.id = rc.lease_id
        WHERE l.landlord_user_id = $1
        ORDER BY rc.due_date DESC, rc.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_landlord_payouts(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<PayoutView>>, AppError> {
    ensure_landlord(&user)?;
    let items = sqlx::query_as::<_, PayoutView>(
        r#"
        SELECT
            id, recipient_user_id, recipient_role, transaction_id, amount, currency,
            status, requested_at, paid_at, failure_reason
        FROM payouts
        WHERE recipient_user_id = $1 AND recipient_role = 'landlord'
        ORDER BY requested_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_landlord_maintenance(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<MaintenanceView>>, AppError> {
    ensure_landlord(&user)?;
    let items = sqlx::query_as::<_, MaintenanceView>(
        r#"
        SELECT
            id, property_id, unit_id, tenant_user_id, landlord_user_id, title,
            description, severity, status, assigned_vendor_name, scheduled_for,
            estimated_cost, actual_cost, created_at, updated_at
        FROM maintenance_requests
        WHERE landlord_user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_landlord_calendar(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<CalendarEventView>>, AppError> {
    ensure_landlord(&user)?;
    let items = sqlx::query_as::<_, CalendarEventView>(
        r#"
        SELECT
            id, user_id, property_id, unit_id, type AS event_type, title,
            starts_at, ends_at, status, metadata_json, created_at
        FROM calendar_events
        WHERE user_id = $1
        ORDER BY starts_at ASC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn admin_metrics_overview(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<AdminOverviewMetrics>, AppError> {
    ensure_admin(&user)?;
    let read_pool = state.read_pool();
    let total_properties = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM properties")
        .fetch_one(read_pool)
        .await?;
    let active_users =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE is_banned = FALSE")
            .fetch_one(read_pool)
            .await?;
    let monthly_revenue = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT COALESCE(SUM(amount), 0)::bigint FROM transactions WHERE status = 'succeeded' AND created_at >= date_trunc('month', NOW())",
    )
    .fetch_one(read_pool)
    .await?
    .unwrap_or(0);
    let open_disputes = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM disputes WHERE status IN ('open', 'in_review', 'escalated')",
    )
    .fetch_one(read_pool)
    .await?;
    Ok(Json(AdminOverviewMetrics {
        total_properties,
        active_users,
        monthly_revenue,
        open_disputes,
    }))
}

pub async fn admin_need_analytics(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<AdminNeedAnalytics>, AppError> {
    ensure_admin(&user)?;
    let read_pool = state.read_pool();
    let total_needs = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts")
        .fetch_one(read_pool)
        .await?;
    let answered_needs = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT post_id) FROM responses WHERE post_id IS NOT NULL",
    )
    .fetch_one(read_pool)
    .await?;
    let response_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM responses")
        .fetch_one(read_pool)
        .await?;
    let open_needs = total_needs.saturating_sub(answered_needs);
    let answer_rate = if total_needs > 0 {
        (answered_needs as f64 / total_needs as f64) * 100.0
    } else {
        0.0
    };
    let monthly_trend = sqlx::query_scalar::<_, Value>(
        r#"
        WITH months AS (
            SELECT generate_series(
                date_trunc('month', NOW()) - interval '7 months',
                date_trunc('month', NOW()),
                interval '1 month'
            ) AS month_start
        ),
        needs AS (
            SELECT date_trunc('month', created_at) AS month_start, COUNT(*)::bigint AS needs_created
            FROM posts
            GROUP BY 1
        ),
        answers AS (
            SELECT date_trunc('month', created_at) AS month_start, COUNT(DISTINCT post_id)::bigint AS needs_answered
            FROM responses
            WHERE post_id IS NOT NULL
            GROUP BY 1
        )
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'month', to_char(months.month_start, 'Mon'),
                    'needsCreated', COALESCE(needs.needs_created, 0),
                    'needsAnswered', COALESCE(answers.needs_answered, 0)
                )
                ORDER BY months.month_start
            ),
            '[]'::jsonb
        )
        FROM months
        LEFT JOIN needs ON needs.month_start = months.month_start
        LEFT JOIN answers ON answers.month_start = months.month_start
        "#,
    )
    .fetch_one(read_pool)
    .await?;
    let monthly_trend: Vec<AdminNeedTrendPoint> = serde_json::from_value(monthly_trend)
        .map_err(|error| AppError::internal(error.to_string()))?;

    Ok(Json(AdminNeedAnalytics {
        total_needs,
        answered_needs,
        open_needs,
        response_count,
        answer_rate,
        monthly_trend,
    }))
}

pub async fn admin_list_verifications(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<AdminVerificationQueueItem>>, AppError> {
    ensure_admin(&user)?;
    let read_pool = state.read_pool();
    let pagination = Pagination::try_from(params)?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM verifications")
        .fetch_one(read_pool)
        .await?;
    let items = sqlx::query_as::<_, AdminVerificationQueueItem>(
        r#"
        SELECT
            v.id,
            v.user_id,
            u.full_name AS user_name,
            u.email AS user_email,
            u.role::text AS user_role,
            v.status,
            COALESCE(property_counts.property_count, 0) AS property_count,
            COALESCE(document_types.document_types, ARRAY[]::text[]) AS document_types,
            COALESCE(documents.documents, '[]'::jsonb) AS documents,
            v.submitted_at,
            v.reviewed_at,
            v.rejection_reason,
            v.notes,
            v.created_at,
            v.updated_at
        FROM verifications v
        INNER JOIN users u ON u.id = v.user_id
        LEFT JOIN (
            SELECT vd.verification_id, ARRAY_AGG(vd.document_type ORDER BY vd.created_at ASC) AS document_types
            FROM verification_documents vd
            GROUP BY vd.verification_id
        ) document_types ON document_types.verification_id = v.id
        LEFT JOIN (
            SELECT
                vd.verification_id,
                jsonb_agg(
                    jsonb_build_object(
                        'id', vd.id,
                        'documentType', vd.document_type,
                        'fileUrl', vd.file_url,
                        'fileKey', vd.file_key,
                        'mimeType', vd.mime_type,
                        'status', vd.status,
                        'createdAt', vd.created_at
                    )
                    ORDER BY vd.created_at ASC
                ) AS documents
            FROM verification_documents vd
            GROUP BY vd.verification_id
        ) documents ON documents.verification_id = v.id
        LEFT JOIN (
            SELECT user_id, SUM(property_count)::bigint AS property_count
            FROM (
                SELECT owner_id AS user_id, COUNT(*)::bigint AS property_count
                FROM properties
                GROUP BY owner_id
                UNION ALL
                SELECT agent_id AS user_id, COUNT(*)::bigint AS property_count
                FROM properties
                WHERE agent_id IS NOT NULL
                GROUP BY agent_id
            ) property_counts_union
            GROUP BY user_id
        ) property_counts ON property_counts.user_id = v.user_id
        ORDER BY v.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(read_pool)
    .await?;
    Ok(Json(PaginatedResponse {
        items,
        total,
        page: pagination.page(),
        per_page: pagination.per_page(),
    }))
}

pub async fn admin_get_verification_detail(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(verification_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    ensure_admin(&user)?;
    let read_pool = state.read_pool();

    let verification = sqlx::query(
        r#"
        SELECT
            v.id,
            v.user_id,
            u.full_name AS user_name,
            u.email AS user_email,
            u.role::text AS user_role,
            v.status,
            v.submitted_at,
            v.reviewed_at,
            v.rejection_reason,
            v.notes,
            v.created_at,
            v.updated_at,
            COALESCE(property_counts.property_count, 0) AS property_count
        FROM verifications v
        INNER JOIN users u ON u.id = v.user_id
        LEFT JOIN (
            SELECT user_id, SUM(property_count)::bigint AS property_count
            FROM (
                SELECT owner_id AS user_id, COUNT(*)::bigint AS property_count
                FROM properties
                GROUP BY owner_id
                UNION ALL
                SELECT agent_id AS user_id, COUNT(*)::bigint AS property_count
                FROM properties
                WHERE agent_id IS NOT NULL
                GROUP BY agent_id
            ) property_counts_union
            GROUP BY user_id
        ) property_counts ON property_counts.user_id = v.user_id
        WHERE v.id = $1
        "#,
    )
    .bind(verification_id)
    .fetch_optional(read_pool)
    .await?
    .ok_or_else(|| AppError::not_found("verification not found"))?;

    let documents = fetch_verification_documents(read_pool, verification_id).await?;

    Ok(Json(json!({
        "verification": {
            "id": verification.get::<Uuid, _>("id"),
            "userId": verification.get::<Uuid, _>("user_id"),
            "userName": verification.get::<String, _>("user_name"),
            "userEmail": verification.get::<String, _>("user_email"),
            "userRole": verification.get::<String, _>("user_role"),
            "status": verification.get::<String, _>("status"),
            "submittedAt": verification.try_get::<Option<DateTime<Utc>>, _>("submitted_at")?,
            "reviewedAt": verification.try_get::<Option<DateTime<Utc>>, _>("reviewed_at")?,
            "rejectionReason": verification.try_get::<Option<String>, _>("rejection_reason")?,
            "notes": verification.try_get::<Option<String>, _>("notes")?,
            "createdAt": verification.get::<DateTime<Utc>, _>("created_at"),
            "updatedAt": verification.get::<DateTime<Utc>, _>("updated_at"),
            "propertyCount": verification.get::<i64, _>("property_count")
        },
        "documents": documents
    })))
}

pub async fn admin_update_verification(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(verification_id): Path<Uuid>,
    Json(payload): Json<UpdateAgentVerificationInput>,
) -> Result<Json<VerificationView>, AppError> {
    ensure_admin(&user)?;
    let mapped_status = match payload.verification_status.as_str() {
        "pending" => "in_review",
        "verified" => "approved",
        "rejected" => "rejected",
        other => other,
    };
    if !matches!(
        mapped_status,
        "submitted" | "in_review" | "approved" | "rejected" | "expired"
    ) {
        return Err(AppError::bad_request("invalid verification_status"));
    }

    let verification = sqlx::query_as::<_, VerificationView>(
        r#"
        UPDATE verifications
        SET status = $2,
            reviewed_at = NOW(),
            reviewed_by = $3,
            rejection_reason = CASE WHEN $2 = 'rejected' THEN $4 ELSE NULL END,
            notes = COALESCE($5, notes),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, user_id, status, submitted_at, reviewed_at, reviewed_by, rejection_reason,
                  notes, created_at, updated_at
        "#,
    )
    .bind(verification_id)
    .bind(mapped_status)
    .bind(user.id)
    .bind(payload.verification_notes.as_deref())
    .bind(payload.verification_notes.as_deref())
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("verification not found"))?;

    let user_record = state
        .user_repository
        .find_by_id(verification.user_id)
        .await?
        .ok_or_else(|| AppError::not_found("user not found"))?;

    let legacy_status = match mapped_status {
        "approved" => "verified",
        "rejected" => "rejected",
        "in_review" | "submitted" => "pending",
        _ => "pending",
    };
    sqlx::query(
        r#"
        UPDATE users
        SET verification_status = $2,
            verification_notes = $3,
            verified_at = CASE WHEN $2 = 'verified' THEN NOW() ELSE verified_at END,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(user_record.id)
    .bind(legacy_status)
    .bind(payload.verification_notes.as_deref())
    .execute(&state.pool)
    .await?;

    let email = state.mail_service.kyc_status_email(
        user_record.email.clone(),
        &user_record.full_name,
        legacy_status,
        payload.verification_notes.as_deref(),
        &kyc_header_asset(legacy_status),
    );
    state.mail_service.send(email).await?;

    insert_notification(
        &state.pool,
        user_record.id,
        if legacy_status == "verified" {
            "kyc_approved"
        } else if legacy_status == "rejected" {
            "kyc_rejected"
        } else {
            "kyc_updated"
        },
        if legacy_status == "verified" {
            "KYC approved"
        } else if legacy_status == "rejected" {
            "KYC rejected"
        } else {
            "KYC updated"
        },
        if legacy_status == "verified" {
            "Your identity verification was approved."
        } else if legacy_status == "rejected" {
            "Your identity verification was rejected. Review the notes and resubmit."
        } else {
            "Your identity verification status changed."
        },
        Some(match user_record.role {
            UserRole::Agent if legacy_status == "rejected" => "/onboarding",
            UserRole::Landlord if legacy_status == "rejected" => "/onboarding",
            UserRole::Agent => "/provider/settings",
            UserRole::Landlord => "/landlord/settings",
            UserRole::Seeker => "/seeker/settings",
            UserRole::Admin => "/admin/settings",
            UserRole::Unassigned => "/onboarding",
        }),
        json!({
            "verificationId": verification.id,
            "status": legacy_status,
            "notes": payload.verification_notes
        }),
    )
    .await?;

    Ok(Json(verification))
}

pub async fn seeker_dashboard_overview(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden(
            "only seekers can view seeker dashboard",
        ));
    }

    // Check verification status for seekers
    let verification_status = sqlx::query_scalar::<_, Option<String>>(
        "SELECT verification_status FROM users WHERE id = $1",
    )
    .bind(&user.id)
    .fetch_one(&state.pool)
    .await?;

    if verification_status.as_deref() != Some("verified")
        && verification_status.as_deref() != Some("approved")
    {
        return Err(AppError::forbidden(
            "please complete your facial verification to access the dashboard",
        ));
    }

    let need_count =
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM posts WHERE author_id = $1 AND status = 'active'",
        )
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
    let saved_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM saved_properties WHERE user_id = $1")
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
    let booking_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bookings WHERE seeker_user_id = $1")
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
    let recent_offers = list_seeker_offers(State(state.clone()), AuthUser(user.clone()))
        .await?
        .0;
    let saved_properties = list_saved_properties(State(state.clone()), AuthUser(user.clone()))
        .await?
        .0;

    Ok(Json(json!({
        "stats": {
            "needCount": need_count,
            "savedCount": saved_count,
            "bookingCount": booking_count
        },
        "matchTrends": [],
        "savedProperties": saved_properties,
        "recentOffers": recent_offers
    })))
}

pub async fn agent_dashboard_overview(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role != UserRole::Agent {
        return Err(AppError::forbidden("only agents can view agent dashboard"));
    }

    // Check verification status for agents
    let verification_status = sqlx::query_scalar::<_, Option<String>>(
        "SELECT verification_status FROM users WHERE id = $1",
    )
    .bind(&user.id)
    .fetch_one(&state.pool)
    .await?;

    if verification_status.as_deref() != Some("verified")
        && verification_status.as_deref() != Some("approved")
    {
        return Err(AppError::forbidden(
            "please complete your identity verification and facial verification to access the dashboard",
        ));
    }

    let listing_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM properties WHERE owner_id = $1 OR agent_id = $1",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let lead_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM agent_post_notifications WHERE agent_id = $1",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let payout_total = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT COALESCE(SUM(amount), 0)::bigint FROM payouts WHERE recipient_user_id = $1 AND recipient_role = 'agent'",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(0);
    let top_listings = list_agent_properties(State(state.clone()), AuthUser(user.clone()))
        .await?
        .0;
    let recent_leads = list_agent_leads(State(state), AuthUser(user)).await?.0;

    Ok(Json(json!({
        "stats": {
            "listingCount": listing_count,
            "leadCount": lead_count,
            "payoutTotal": payout_total
        },
        "earningsSeries": [],
        "topListings": top_listings,
        "recentLeads": recent_leads
    })))
}

pub async fn landlord_dashboard_overview(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    ensure_landlord(&user)?;

    // Check verification status for landlords
    let verification_status = sqlx::query_scalar::<_, Option<String>>(
        "SELECT verification_status FROM users WHERE id = $1",
    )
    .bind(&user.id)
    .fetch_one(&state.pool)
    .await?;

    // Allow access if verification is verified, approved, or pending (in progress)
    if verification_status.as_deref() != Some("verified")
        && verification_status.as_deref() != Some("approved")
        && verification_status.as_deref() != Some("pending")
    {
        return Err(AppError::forbidden(
            "please complete your identity verification to access the dashboard",
        ));
    }

    let property_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM properties WHERE owner_id = $1")
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
    let unit_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM units u INNER JOIN properties p ON p.id = u.property_id WHERE p.owner_id = $1",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let open_maintenance = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM maintenance_requests WHERE landlord_user_id = $1 AND status IN ('open','assigned','in_progress')",
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let maintenance_queue = list_landlord_maintenance(State(state.clone()), AuthUser(user.clone()))
        .await?
        .0;

    Ok(Json(json!({
        "stats": {
            "propertyCount": property_count,
            "unitCount": unit_count,
            "openMaintenance": open_maintenance
        },
        "occupancySeries": [],
        "collectionSeries": [],
        "leaseExpiries": [],
        "maintenanceQueue": maintenance_queue
    })))
}

pub async fn get_agent_lead_detail(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if user.role != UserRole::Agent {
        return Err(AppError::forbidden("only agents can view leads"));
    }
    let lead = sqlx::query_as::<_, AgentLeadView>(
        r#"
        SELECT
            COALESCE(lm.id, apn.id) AS id,
            p.id AS need_post_id,
            lm.matched_property_id,
            p.target_property_id,
            p.target_property_title,
            p.target_property_image_url,
            p.target_property_location,
            COALESCE(lm.match_score, 0)::double precision AS match_score,
            COALESCE(lm.status, CASE WHEN apn.is_read THEN 'viewed' ELSE 'new' END) AS status,
            lm.sla_expires_at,
            COALESCE(lm.created_at, apn.created_at) AS created_at,
            COALESCE(lm.updated_at, apn.created_at) AS updated_at,
            p.request_title,
            p.location,
            p.property_type,
            NULL::text AS urgency
        FROM agent_post_notifications apn
        INNER JOIN posts p ON p.id = apn.post_id
        LEFT JOIN lead_matches lm ON lm.agent_user_id = apn.agent_id AND lm.need_post_id = apn.post_id
        WHERE apn.agent_id = $1 AND COALESCE(lm.id, apn.id) = $2
        "#,
    )
    .bind(user.id)
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("lead not found"))?;

    let need = sqlx::query(
        r#"
        SELECT to_jsonb(p) AS payload
        FROM posts p
        WHERE p.id = $1
        "#,
    )
    .bind(lead.need_post_id)
    .fetch_one(&state.pool)
    .await?;
    let matched_properties = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(to_jsonb(x)), '[]'::jsonb)
        FROM (
            SELECT p.id, p.title, p.location, p.price, p.images
            FROM properties p
            WHERE (p.owner_id = $1 OR p.agent_id = $1)
        ) x
        "#,
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let existing_offer = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT to_jsonb(o)
        FROM offers o
        WHERE o.provider_user_id = $1 AND o.need_post_id = $2
        ORDER BY o.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user.id)
    .bind(lead.need_post_id)
    .fetch_optional(&state.pool)
    .await?
    .unwrap_or(Value::Null);

    let need_payload: Value = need.try_get("payload").map_err(anyhow::Error::from)?;
    Ok(Json(json!({
        "lead": lead,
        "seekerNeed": need_payload,
        "matchedProperties": matched_properties,
        "existingOffer": existing_offer
    })))
}

pub async fn update_agent_property(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAgentPropertyInput>,
) -> Result<Json<Value>, AppError> {
    if user.role != UserRole::Agent {
        return Err(AppError::forbidden(
            "only agents can update agent properties",
        ));
    }
    // Load current property
    let current = state.property_use_cases.get_by_id(id, Some(&user)).await?;
    let current_status = String::from(match current.status {
        crate::domain::properties::PropertyStatus::Draft => "draft",
        crate::domain::properties::PropertyStatus::PendingVerification => "pending_verification",
        crate::domain::properties::PropertyStatus::Verified => "verified",
        crate::domain::properties::PropertyStatus::Published => "published",
        crate::domain::properties::PropertyStatus::Hidden => "hidden",
        crate::domain::properties::PropertyStatus::Suspended => "suspended",
        crate::domain::properties::PropertyStatus::RentedOut => "rented_out",
        crate::domain::properties::PropertyStatus::SoldOut => "sold_out",
        crate::domain::properties::PropertyStatus::InUse => "in_use",
    });
    let requested_status = payload
        .status
        .as_ref()
        .map(|value| value.trim().to_lowercase());

    if let Some(locked_until) = current.status_locked_until {
        if locked_until > Utc::now() {
            if let Some(next_status) = &requested_status {
                if next_status != &current_status {
                    return Err(AppError::bad_request(&format!(
                        "This property is locked in its current status until {}.",
                        locked_until.format("%d %b %Y %I:%M %p WAT")
                    )));
                }
            }
        }
    }

    let timed_statuses = ["hidden", "rented_out", "in_use"];
    let parsed_available_at = if let Some(status) = requested_status.as_deref() {
        if timed_statuses.contains(&status) {
            let available_at_str = payload.available_at.as_deref().ok_or_else(|| {
                AppError::bad_request(
                    "Timed status changes require an availability end date.",
                )
            })?;
            let available_at = chrono::DateTime::parse_from_rfc3339(available_at_str)
                .map_err(|_| AppError::bad_request("Invalid availability date supplied."))?
                .with_timezone(&Utc);
            if available_at <= Utc::now() {
                return Err(AppError::bad_request(
                    "Availability end date must be in the future.",
                ));
            }
            Some(available_at)
        } else {
            None
        }
    } else {
        None
    };

    // Media deletion validation
    let old_images = &current.images;
    let new_images = payload.images.clone().unwrap_or_else(|| old_images.clone());
    let existing_count = old_images.len();
    let removed_existing = old_images.iter().filter(|img| !new_images.contains(img)).count();
    let final_count = new_images.len();
    let max_remove = std::cmp::max(1, existing_count / 3);
    if final_count == 0 {
        return Err(AppError::bad_request("You must have at least one image."));
    }
    if removed_existing > max_remove {
        return Err(AppError::bad_request(&format!(
            "You can only remove up to {} of the current images in one edit.", max_remove
        )));
    }

    // Price logic
    let old_price = current.price;
    let new_price = payload.price.unwrap_or(old_price);
    let mut price_change_pending = false;
    let mut saved_fields = vec![];
    let mut message = String::new();

    if new_price > old_price {
        // Save all other fields except price
        let title = payload.title.clone();
        let description = payload.description.clone();
        let location = payload.location.clone();
        let exact_address = payload.exact_address.clone();
        let contact_name = payload.contact_name.clone();
        let contact_phone = payload.contact_phone.clone();
        let listing_type = payload.listing_type.clone();
        let status = payload.status.clone();
        sqlx::query(
            r#"
            UPDATE properties
            SET title = COALESCE($3, title),
                description = COALESCE($4, description),
                location = COALESCE($5, location),
                exact_address = COALESCE($6, exact_address),
                images = COALESCE($7, images),
                contact_name = COALESCE($8, contact_name),
                contact_phone = COALESCE($9, contact_phone),
                listing_type = COALESCE($10, listing_type),
                status = COALESCE($11::property_status, status),
                updated_at = NOW()
            WHERE id = $1 AND (owner_id = $2 OR agent_id = $2)
            "#,
        )
        .bind(id)
        .bind(user.id)
        .bind(title)
        .bind(description)
        .bind(location)
        .bind(exact_address)
        .bind(Some(new_images.clone()))
        .bind(contact_name)
        .bind(contact_phone)
        .bind(listing_type)
        .bind(&status)
        .execute(&state.pool)
        .await?;
        // Create or update pending price change request
        let payload_json = serde_json::to_value(&payload).unwrap();
        sqlx::query(
            r#"
            INSERT INTO property_change_requests (
                id, property_id, requested_by, request_type, current_price, requested_price, status, old_value_json, new_value_json, created_at
            ) VALUES ($1, $2, $3, 'price_increase', $4, $5, 'pending', $6, $7, NOW())
            ON CONFLICT (property_id, request_type) WHERE status = 'pending'
            DO UPDATE SET requested_price = $5, new_value_json = $7, created_at = NOW()
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(id)
        .bind(user.id)
        .bind(old_price)
        .bind(new_price)
        .bind(serde_json::to_value(&current).unwrap())
        .bind(payload_json)
        .execute(&state.pool)
        .await?;
        price_change_pending = true;
        message = "Your other changes were saved. Price increase is pending admin approval.".to_string();
        saved_fields = vec!["title","description","location","exact_address","images","contact_name","contact_phone","listing_type","status"];
    } else {
        // Save all fields including price
        let title = payload.title.clone();
        let description = payload.description.clone();
        let location = payload.location.clone();
        let exact_address = payload.exact_address.clone();
        let contact_name = payload.contact_name.clone();
        let contact_phone = payload.contact_phone.clone();
        let listing_type = payload.listing_type.clone();
        let status = payload.status.clone();
        sqlx::query(
            r#"
            UPDATE properties
            SET title = COALESCE($3, title),
                description = COALESCE($4, description),
                price = COALESCE($5, price),
                location = COALESCE($6, location),
                exact_address = COALESCE($7, exact_address),
                images = COALESCE($8, images),
                contact_name = COALESCE($9, contact_name),
                contact_phone = COALESCE($10, contact_phone),
                listing_type = COALESCE($11, listing_type),
                status = COALESCE($12::property_status, status),
                updated_at = NOW()
            WHERE id = $1 AND (owner_id = $2 OR agent_id = $2)
            "#,
        )
        .bind(id)
        .bind(user.id)
        .bind(title)
        .bind(description)
        .bind(Some(new_price))
        .bind(location)
        .bind(exact_address)
        .bind(Some(new_images.clone()))
        .bind(contact_name)
        .bind(contact_phone)
        .bind(listing_type)
        .bind(&status)
        .execute(&state.pool)
        .await?;
        message = "Property updated successfully.".to_string();
        saved_fields = vec!["title","description","price","location","exact_address","images","contact_name","contact_phone","listing_type","status"];
    }

    if let Some(available_at) = parsed_available_at {
        sqlx::query(
            r#"
            INSERT INTO property_rental_periods (id, property_id, available_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(id)
        .bind(available_at)
        .execute(&state.pool)
        .await?;
    }

    let detail = state.property_use_cases.get_by_id(id, Some(&user)).await?;
    Ok(Json(json!({
        "property": detail,
        "priceChangePending": price_change_pending,
        "savedFields": saved_fields,
        "message": message
    })))
}

pub async fn list_agent_payouts(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<PayoutView>>, AppError> {
    if user.role != UserRole::Agent {
        return Err(AppError::forbidden("only agents can view payouts"));
    }
    let items = sqlx::query_as::<_, PayoutView>(
        r#"
        SELECT id, recipient_user_id, recipient_role, transaction_id, amount, currency,
               status, requested_at, paid_at, failure_reason
        FROM payouts
        WHERE recipient_user_id = $1 AND recipient_role = 'agent'
        ORDER BY requested_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_agent_calendar(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<Value>>, AppError> {
    if user.role != UserRole::Agent {
        return Err(AppError::forbidden("only agents can view calendar"));
    }
    let events = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(to_jsonb(x)), '[]'::jsonb)
        FROM (
            SELECT
                b.id,
                'booking'::text AS "eventType",
                COALESCE(p.title, 'Booking') AS title,
                b.scheduled_for AS "startsAt",
                b.scheduled_for AS "endsAt",
                b.status,
                jsonb_build_object('bookingType', b.booking_type, 'propertyId', b.property_id) AS metadata
            FROM bookings b
            LEFT JOIN properties p ON p.id = b.property_id
            WHERE b.provider_user_id = $1
            ORDER BY b.scheduled_for ASC
        ) x
        "#,
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let items: Vec<Value> = serde_json::from_value(events).map_err(anyhow::Error::from)?;
    Ok(Json(items))
}

pub async fn create_landlord_property(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<crate::domain::properties::CreatePropertyInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    ensure_landlord(&user)?;
    let detail = state.property_use_cases.create(&user, payload).await?;
    Ok((StatusCode::CREATED, Json(json!(detail))))
}

pub async fn create_landlord_unit(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateUnitInput>,
) -> Result<(StatusCode, Json<UnitView>), AppError> {
    ensure_landlord(&user)?;
    let owns_property = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM properties WHERE id = $1 AND owner_id = $2)",
    )
    .bind(payload.property_id)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    if !owns_property {
        return Err(AppError::forbidden("property does not belong to landlord"));
    }
    let unit = sqlx::query_as::<_, UnitView>(
        r#"
        INSERT INTO units (
            id, property_id, unit_code, name, unit_type, bedrooms_label, rent_amount,
            rent_currency, rent_period, occupancy_status, listing_status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, 'NGN'), COALESCE($9, 'year'),
                COALESCE($10, 'vacant'), COALESCE($11, 'unlisted'))
        RETURNING id, property_id, unit_code, name, unit_type, bedrooms_label, rent_amount,
                  rent_currency, rent_period, occupancy_status, listing_status, tenant_user_id,
                  lease_id, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.property_id)
    .bind(payload.unit_code)
    .bind(payload.name)
    .bind(payload.unit_type)
    .bind(payload.bedrooms_label)
    .bind(payload.rent_amount)
    .bind(payload.rent_currency)
    .bind(payload.rent_period)
    .bind(payload.occupancy_status)
    .bind(payload.listing_status)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(unit)))
}

pub async fn create_landlord_maintenance(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateMaintenanceRequestInput>,
) -> Result<(StatusCode, Json<MaintenanceView>), AppError> {
    ensure_landlord(&user)?;
    let owns_property = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM properties WHERE id = $1 AND owner_id = $2)",
    )
    .bind(payload.property_id)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    if !owns_property {
        return Err(AppError::forbidden("property does not belong to landlord"));
    }
    let item = sqlx::query_as::<_, MaintenanceView>(
        r#"
        INSERT INTO maintenance_requests (
            id, property_id, unit_id, landlord_user_id, title, description, severity,
            status, scheduled_for, estimated_cost, reported_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'open', $8, $9, $4)
        RETURNING id, property_id, unit_id, tenant_user_id, landlord_user_id, title,
                  description, severity, status, assigned_vendor_name, scheduled_for,
                  estimated_cost, actual_cost, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.property_id)
    .bind(payload.unit_id)
    .bind(user.id)
    .bind(payload.title)
    .bind(payload.description)
    .bind(payload.severity)
    .bind(payload.scheduled_for)
    .bind(payload.estimated_cost)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn uploads_presign(
    Json(payload): Json<UploadPresignInput>,
) -> Result<Json<Value>, AppError> {
    let file_key = format!(
        "{}/{}-{}",
        payload.category,
        Uuid::new_v4(),
        payload.filename
    );
    let file_url = format!("https://uploads.verinest.local/{}", file_key);
    let upload_url = format!("https://uploads.verinest.local/presigned/{}", file_key);
    Ok(Json(json!({
        "uploadUrl": upload_url,
        "fileUrl": file_url,
        "fileKey": file_key,
        "contentType": payload.content_type
    })))
}

pub async fn list_admin_users(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Value>>, AppError> {
    ensure_admin(&user)?;
    let read_pool = state.read_pool();
    let pagination = Pagination::try_from(params)?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(read_pool)
        .await?;
    let items = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(to_jsonb(x)), '[]'::jsonb)
        FROM (
            SELECT
                u.id,
                u.full_name,
                u.email,
                u.role::text AS role,
                u.email_verified,
                u.verification_status,
                u.is_banned,
                u.created_at,
                p.avatar_url
            FROM users u
            LEFT JOIN profiles p ON p.user_id = u.id
            ORDER BY u.created_at DESC
            LIMIT $1 OFFSET $2
        ) x
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_one(read_pool)
    .await?;
    let items: Vec<Value> = serde_json::from_value(items)
        .map_err(|error| AppError::internal(error.to_string()))?;
    Ok(Json(PaginatedResponse {
        items,
        total,
        page: pagination.page(),
        per_page: pagination.per_page(),
    }))
}

pub async fn admin_suspend_user(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<Value>, AppError> {
    ensure_admin(&user)?;
    sqlx::query("UPDATE users SET is_banned = TRUE WHERE id = $1")
        .bind(&user_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(serde_json::json!({ "success": true, "message": "User suspended" })))
}

pub async fn admin_unsuspend_user(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<Value>, AppError> {
    ensure_admin(&user)?;
    sqlx::query("UPDATE users SET is_banned = FALSE WHERE id = $1")
        .bind(&user_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(serde_json::json!({ "success": true, "message": "User unsuspended" })))
}

pub async fn list_admin_properties(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Value>>, AppError> {
    ensure_admin(&user)?;
    let read_pool = state.read_pool();
    let pagination = Pagination::try_from(params)?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM properties")
        .fetch_one(read_pool)
        .await?;
    let items = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(to_jsonb(x)), '[]'::jsonb)
        FROM (
            SELECT
                p.id,
                p.owner_id,
                p.agent_id,
                p.title,
                p.location,
                p.price,
                p.status::text AS status,
                p.created_at,
                p.is_service_apartment,
                p.listing_type,
                owner.full_name AS owner_name,
                owner_profile.avatar_url AS owner_avatar_url,
                agent.full_name AS agent_name,
                agent_profile.avatar_url AS agent_avatar_url,
                COALESCE(report_stats.report_count, 0) AS report_count,
                COALESCE(report_stats.open_report_count, 0) AS open_report_count
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN profiles owner_profile ON owner_profile.user_id = owner.id
            LEFT JOIN users agent ON agent.id = p.agent_id
            LEFT JOIN profiles agent_profile ON agent_profile.user_id = agent.id
            LEFT JOIN (
                SELECT
                    property_id,
                    COUNT(*)::bigint AS report_count,
                    COUNT(*) FILTER (WHERE status = 'open')::bigint AS open_report_count
                FROM reports
                WHERE property_id IS NOT NULL
                GROUP BY property_id
            ) report_stats ON report_stats.property_id = p.id
            ORDER BY p.created_at DESC
            LIMIT $1 OFFSET $2
        ) x
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_one(read_pool)
    .await?;
    let items: Vec<Value> = serde_json::from_value(items)
        .map_err(|error| AppError::internal(error.to_string()))?;
    Ok(Json(PaginatedResponse {
        items,
        total,
        page: pagination.page(),
        per_page: pagination.per_page(),
    }))
}

pub async fn delete_admin_property(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(property_id): Path<Uuid>,
    Json(payload): Json<AdminDeletePropertyInput>,
) -> Result<Json<Value>, AppError> {
    ensure_admin(&user)?;

    let admin = state
        .user_repository
        .find_by_id(user.id)
        .await?
        .ok_or_else(|| AppError::not_found("admin not found"))?;

    let password_service = PasswordService;
    if !password_service.verify_password(payload.password.trim(), &admin.password_hash)? {
        return Err(AppError::forbidden("invalid password"));
    }

    let property = sqlx::query_scalar::<_, Value>(
        r#"
        WITH deleted AS (
            DELETE FROM properties
            WHERE id = $1
            RETURNING id, title, owner_id, agent_id
        )
        SELECT to_jsonb(deleted)
        FROM deleted
        "#,
    )
    .bind(property_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("property not found"))?;

    state
        .audit_service
        .record(
            AuditActor {
                user_id: Some(admin.id),
                email: Some(admin.email.clone()),
                role: Some("admin".to_string()),
            },
            AuditEvent {
                request_id: Uuid::new_v4(),
                action: "admin.delete_property".to_string(),
                method: "DELETE".to_string(),
                path: format!("/admin/properties/{property_id}"),
                status_code: StatusCode::OK.as_u16(),
                ip_address: None,
                user_agent: None,
                resource_type: Some("property".to_string()),
                resource_id: Some(property_id),
                success: true,
                metadata: json!({
                    "property": property,
                }),
            },
        )
        .await
        .map_err(anyhow::Error::from)?;

    Ok(Json(json!({
        "success": true,
        "message": "Property deleted successfully"
    })))
}

pub async fn list_admin_transactions(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Value>>, AppError> {
    ensure_admin(&user)?;
    let read_pool = state.read_pool();
    let pagination = Pagination::try_from(params)?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM transactions")
        .fetch_one(read_pool)
        .await?;
    let items = sqlx::query_scalar::<_, Value>(
        "SELECT COALESCE(jsonb_agg(to_jsonb(t)), '[]'::jsonb) FROM (SELECT * FROM transactions ORDER BY created_at DESC LIMIT $1 OFFSET $2) t",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_one(read_pool)
    .await?;
    let items: Vec<Value> = serde_json::from_value(items)
        .map_err(|error| AppError::internal(error.to_string()))?;
    Ok(Json(PaginatedResponse {
        items,
        total,
        page: pagination.page(),
        per_page: pagination.per_page(),
    }))
}

pub async fn list_admin_disputes(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Value>>, AppError> {
    ensure_admin(&user)?;
    let read_pool = state.read_pool();
    let pagination = Pagination::try_from(params)?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM disputes")
        .fetch_one(read_pool)
        .await?;
    let items = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(to_jsonb(d)), '[]'::jsonb)
        FROM (
            SELECT
                disputes.*,
                reporter.full_name AS reporter_name,
                reporter.email AS reporter_email,
                subject.full_name AS subject_name,
                subject.email AS subject_email,
                property.title AS property_title,
                property.location AS property_location,
                assigned_admin.full_name AS assigned_admin_name,
                booking.scheduled_for AS booking_scheduled_for
            FROM disputes
            LEFT JOIN users reporter ON reporter.id = disputes.reporter_user_id
            LEFT JOIN users subject ON subject.id = disputes.subject_user_id
            LEFT JOIN users assigned_admin ON assigned_admin.id = disputes.assigned_admin_id
            LEFT JOIN properties property ON property.id = disputes.property_id
            LEFT JOIN bookings booking ON booking.id = disputes.booking_id
            ORDER BY disputes.created_at DESC
            LIMIT $1 OFFSET $2
        ) d
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_one(read_pool)
    .await?;
    let items: Vec<Value> = serde_json::from_value(items)
        .map_err(|error| AppError::internal(error.to_string()))?;
    Ok(Json(PaginatedResponse {
        items,
        total,
        page: pagination.page(),
        per_page: pagination.per_page(),
    }))
}

pub async fn list_admin_reports(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Value>>, AppError> {
    ensure_admin(&user)?;
    let read_pool = state.read_pool();
    let pagination = Pagination::try_from(params)?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reports")
        .fetch_one(read_pool)
        .await?;
    let items = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(to_jsonb(r)), '[]'::jsonb)
        FROM (
            SELECT
                reports.*,
                property.title AS property_title,
                property.location AS property_location,
                property.status::text AS property_status,
                reporter.full_name AS reporter_name,
                reporter.email AS reporter_email,
                reported_user.full_name AS reported_user_name,
                COALESCE(provider.full_name, owner.full_name) AS property_manager_name,
                COALESCE(provider.email, owner.email) AS property_manager_email
            FROM reports
            LEFT JOIN properties property ON property.id = reports.property_id
            LEFT JOIN users reporter ON reporter.id = reports.reporter_id
            LEFT JOIN users reported_user ON reported_user.id = reports.reported_user_id
            LEFT JOIN users owner ON owner.id = property.owner_id
            LEFT JOIN users provider ON provider.id = property.agent_id
            ORDER BY reports.created_at DESC
            LIMIT $1 OFFSET $2
        ) r
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_one(read_pool)
    .await?;
    let items: Vec<Value> = serde_json::from_value(items)
        .map_err(|error| AppError::internal(error.to_string()))?;
    Ok(Json(PaginatedResponse {
        items,
        total,
        page: pagination.page(),
        per_page: pagination.per_page(),
    }))
}

pub async fn list_admin_announcements(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Value>>, AppError> {
    ensure_admin(&user)?;
    let read_pool = state.read_pool();
    let pagination = Pagination::try_from(params)?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM announcements")
        .fetch_one(read_pool)
        .await?;
    let items = sqlx::query_scalar::<_, Value>(
        "SELECT COALESCE(jsonb_agg(to_jsonb(a)), '[]'::jsonb) FROM (SELECT * FROM announcements ORDER BY created_at DESC LIMIT $1 OFFSET $2) a",
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_one(read_pool)
    .await?;
    let items: Vec<Value> = serde_json::from_value(items)
        .map_err(|error| AppError::internal(error.to_string()))?;
    Ok(Json(PaginatedResponse {
        items,
        total,
        page: pagination.page(),
        per_page: pagination.per_page(),
    }))
}

pub async fn list_public_announcements(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Value>>, AppError> {
    let read_pool = state.read_pool();
    let pagination = Pagination::try_from(params)?;
    let allowed_audiences = announcement_audiences_for_role(user.role);
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM announcements WHERE status = 'published' AND audience = ANY($1)",
    )
    .bind(&allowed_audiences)
    .fetch_one(read_pool)
    .await?;
    let items = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(to_jsonb(a)), '[]'::jsonb)
        FROM (
            SELECT
                announcements.*,
                creator.full_name AS created_by_name,
                creator.role::text AS created_by_role
            FROM announcements
            LEFT JOIN users creator ON creator.id = announcements.created_by
            WHERE announcements.status = 'published'
              AND announcements.audience = ANY($1)
            ORDER BY announcements.published_at DESC NULLS LAST, announcements.created_at DESC
            LIMIT $2 OFFSET $3
        ) a
        "#,
    )
    .bind(&allowed_audiences)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_one(read_pool)
    .await?;
    let items: Vec<Value> = serde_json::from_value(items)
        .map_err(|error| AppError::internal(error.to_string()))?;
    Ok(Json(PaginatedResponse {
        items,
        total,
        page: pagination.page(),
        per_page: pagination.per_page(),
    }))
}

pub async fn get_public_announcement(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let allowed_audiences = announcement_audiences_for_role(user.role);
    let item = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT to_jsonb(a)
        FROM (
            SELECT
                announcements.*,
                creator.full_name AS created_by_name,
                creator.role::text AS created_by_role
            FROM announcements
            LEFT JOIN users creator ON creator.id = announcements.created_by
            WHERE announcements.id = $1
              AND announcements.status = 'published'
              AND announcements.audience = ANY($2)
        ) a
        "#,
    )
    .bind(id)
    .bind(&allowed_audiences)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("announcement not found"))?;

    Ok(Json(item))
}

pub async fn create_admin_announcement(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateAnnouncementInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    ensure_admin(&user)?;
    let item = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT to_jsonb(x)
        FROM (
            INSERT INTO announcements (id, title, body, audience, status, published_at, created_by)
            VALUES ($1, $2, $3, $4, 'published', NOW(), $5)
            RETURNING *
        ) x
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.title.clone())
    .bind(payload.body.clone())
    .bind(payload.audience.clone())
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    let announcement_id = item
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let target_roles: Option<Vec<&'static str>> =
        match payload.audience.trim().to_lowercase().as_str() {
            "all" | "all users" => None,
            "seekers" | "seekers only" | "tenants" => Some(vec!["seeker"]),
            "providers" | "providers only" => Some(vec!["agent", "landlord"]),
            "agents" | "agents only" => Some(vec!["agent"]),
            "landlords" | "landlords only" => Some(vec!["landlord"]),
            _ => None,
        };
    let announcement_action_url = if announcement_id.is_empty() {
        "/announcements".to_string()
    } else {
        format!("/announcements/{announcement_id}")
    };

    let recipients = if let Some(roles) = target_roles {
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE role::text = ANY($1)")
            .bind(&roles)
            .fetch_all(&state.pool)
            .await?
    } else {
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM users")
            .fetch_all(&state.pool)
            .await?
    };

    for recipient_id in recipients {
        if let Err(error) = insert_notification(
            &state.pool,
            recipient_id,
            "announcement",
            &payload.title,
            &payload.body,
            Some(announcement_action_url.as_str()),
            json!({
                "audience": payload.audience,
                "announcementId": item.get("id").cloned().unwrap_or(Value::Null)
            }),
        )
        .await
        {
            tracing::error!("failed to fan out announcement notification: {error:?}");
        }
    }

    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn list_notifications(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Value>>, AppError> {
    let pagination = Pagination::try_from(params)?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notifications WHERE user_id = $1")
        .bind(user.id)
        .fetch_one(&state.pool)
        .await?;
    let items = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(to_jsonb(x)), '[]'::jsonb)
        FROM (
            SELECT id, type AS kind, title, body, data_json->>'actionUrl' AS "actionUrl", read_at AS "readAt", created_at AS "createdAt"
            FROM notifications
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
        ) x
        "#,
    )
    .bind(user.id)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_one(&state.pool)
    .await?;
    let items: Vec<Value> = serde_json::from_value(items)
        .map_err(|error| AppError::internal(error.to_string()))?;
    Ok(Json(PaginatedResponse {
        items,
        total,
        page: pagination.page(),
        per_page: pagination.per_page(),
    }))
}

pub async fn notifications_read_all(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    sqlx::query("UPDATE notifications SET read_at = NOW() WHERE user_id = $1 AND read_at IS NULL")
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn notification_mark_read(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    sqlx::query("UPDATE notifications SET read_at = NOW() WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn notification_delete(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn fetch_profile(pool: &PgPool, user_id: Uuid) -> Result<Option<ProfileView>, AppError> {
    let profile = sqlx::query_as::<_, ProfileView>(
        r#"
        SELECT
            p.id,
            p.user_id,
            p.full_name,
            p.phone,
            p.city,
            COALESCE(NULLIF(TRIM(u.wallet_address), ''), p.avatar_url) AS avatar_url,
            p.bio,
            p.onboarding_completed,
            p.created_at,
            p.updated_at
        FROM profiles p
        JOIN users u ON u.id = p.user_id
        WHERE p.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(profile)
}

async fn fetch_role_profile(pool: &PgPool, user: &User) -> Result<Option<Value>, AppError> {
    let value = match user.role {
        UserRole::Unassigned => None,
        UserRole::Seeker => {
            sqlx::query_scalar::<_, Value>(
                "SELECT to_jsonb(sp) FROM seeker_profiles sp WHERE sp.user_id = $1",
            )
            .bind(user.id)
            .fetch_optional(pool)
            .await?
        }
        UserRole::Agent => {
            sqlx::query_scalar::<_, Value>(
                "SELECT to_jsonb(ap) FROM agent_profiles ap WHERE ap.user_id = $1",
            )
            .bind(user.id)
            .fetch_optional(pool)
            .await?
        }
        UserRole::Landlord => {
            sqlx::query_scalar::<_, Value>(
                "SELECT to_jsonb(lp) FROM landlord_profiles lp WHERE lp.user_id = $1",
            )
            .bind(user.id)
            .fetch_optional(pool)
            .await?
        }
        UserRole::Admin => None,
    };
    Ok(value)
}

async fn fetch_policy_metadata(pool: &PgPool) -> Result<PolicyMetadataView, AppError> {
    sqlx::query(
        r#"
        INSERT INTO site_policy_settings (singleton, terms_version, privacy_version, effective_at, change_summary)
        VALUES (
            TRUE,
            '2026.05',
            '2026.05',
            NOW(),
            'We updated our Terms and Privacy Policy. Please review the latest versions before continuing to use Verinest.'
        )
        ON CONFLICT (singleton) DO NOTHING
        "#,
    )
    .execute(pool)
    .await?;

    let metadata = sqlx::query_as::<_, PolicyMetadataView>(
        r#"
        SELECT terms_version, privacy_version, effective_at, change_summary, updated_at
        FROM site_policy_settings
        WHERE singleton = TRUE
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(metadata)
}

async fn fetch_policy_acceptance(
    pool: &PgPool,
    user_id: Uuid,
    metadata: &PolicyMetadataView,
) -> Result<PolicyAcceptanceView, AppError> {
    let row = sqlx::query(
        r#"
        SELECT terms_version, privacy_version, accepted_at
        FROM user_legal_acceptances
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let terms_version_accepted = row.try_get::<Option<String>, _>("terms_version")?;
        let privacy_version_accepted = row.try_get::<Option<String>, _>("privacy_version")?;
        let accepted_at = row.try_get::<Option<DateTime<Utc>>, _>("accepted_at")?;
        let requires_reacceptance =
            terms_version_accepted.as_deref() != Some(metadata.terms_version.as_str())
                || privacy_version_accepted.as_deref() != Some(metadata.privacy_version.as_str());

        return Ok(PolicyAcceptanceView {
            terms_version_accepted,
            privacy_version_accepted,
            accepted_at,
            requires_reacceptance,
        });
    }

    Ok(PolicyAcceptanceView {
        terms_version_accepted: None,
        privacy_version_accepted: None,
        accepted_at: None,
        requires_reacceptance: true,
    })
}

async fn upsert_user_policy_acceptance(
    pool: &PgPool,
    user_id: Uuid,
    terms_version: &str,
    privacy_version: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO user_legal_acceptances (user_id, terms_version, privacy_version, accepted_at, updated_at)
        VALUES ($1, $2, $3, NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE
        SET terms_version = EXCLUDED.terms_version,
            privacy_version = EXCLUDED.privacy_version,
            accepted_at = NOW(),
            updated_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(terms_version)
    .bind(privacy_version)
    .execute(pool)
    .await?;

    Ok(())
}

async fn fetch_latest_verification(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<VerificationView>, AppError> {
    let verification = sqlx::query_as::<_, VerificationView>(
        r#"
        SELECT id, user_id, status, submitted_at, reviewed_at, reviewed_by, rejection_reason,
               notes, created_at, updated_at
        FROM verifications
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(verification)
}

async fn fetch_verification_documents(
    pool: &PgPool,
    verification_id: Uuid,
) -> Result<Vec<VerificationDocumentView>, AppError> {
    let documents = sqlx::query_as::<_, VerificationDocumentView>(
        r#"
        SELECT id, verification_id, document_type, file_url, file_key, mime_type, status, created_at
        FROM verification_documents
        WHERE verification_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(verification_id)
    .fetch_all(pool)
    .await?;
    Ok(documents)
}

fn ensure_landlord(user: &User) -> Result<(), AppError> {
    if user.role != UserRole::Landlord {
        return Err(AppError::forbidden(
            "only landlords can access this endpoint",
        ));
    }
    Ok(())
}

fn ensure_admin(user: &User) -> Result<(), AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::forbidden("only admins can access this endpoint"));
    }
    Ok(())
}

fn announcement_audiences_for_role(role: UserRole) -> Vec<String> {
    match role {
        UserRole::Seeker => vec!["all".to_string(), "seekers".to_string()],
        UserRole::Agent => vec![
            "all".to_string(),
            "agents".to_string(),
            "providers".to_string(),
        ],
        UserRole::Landlord => vec![
            "all".to_string(),
            "landlords".to_string(),
            "providers".to_string(),
        ],
        UserRole::Admin => vec![
            "all".to_string(),
            "seekers".to_string(),
            "agents".to_string(),
            "landlords".to_string(),
            "admins".to_string(),
            "providers".to_string(),
        ],
        UserRole::Unassigned => vec!["all".to_string()],
    }
}

pub async fn list_public_properties(
    State(state): State<AppState>,
    Query(query): Query<PropertyQuery>,
) -> Result<Json<Vec<PropertyListItem>>, AppError> {
    Ok(Json(state.property_use_cases.list(query).await?))
}

pub async fn list_public_needs(
    State(state): State<AppState>,
    Query(query): Query<PostQuery>,
) -> Result<Json<Vec<PostListItem>>, AppError> {
    Ok(Json(state.post_use_cases.list_posts(query).await?))
}
