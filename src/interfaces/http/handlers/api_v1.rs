use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

use crate::{
    domain::{
        posts::{CreatePostInput, PostListItem, PostQuery},
        properties::{PropertyListItem, PropertyQuery},
        users::{
            AuthResponse, RegisterUserInput, SendEmailCodeInput, UpdateAgentVerificationInput, User,
            UserPublicView, UserRole, VerifyEmailCodeInput,
        },
    },
    interfaces::http::{
        errors::AppError,
        middleware::auth::AuthUser,
        state::AppState,
    },
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiRegisterInput {
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
    pub bio: Option<String>,
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
pub struct UploadPresignInput {
    pub category: String,
    pub filename: String,
    pub content_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgentPropertyInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<i64>,
    pub location: Option<String>,
    pub exact_address: Option<String>,
    pub images: Option<Vec<String>>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthMeResponse {
    pub user: UserPublicView,
    pub profile: Option<ProfileView>,
    pub role_profile: Option<Value>,
    pub verification: Option<VerificationView>,
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

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminVerificationQueueItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub user_role: String,
    pub status: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<ApiRegisterInput>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
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
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn send_email_code(
    State(state): State<AppState>,
    Json(payload): Json<SendEmailCodeInput>,
) -> Result<Json<crate::application::services::ValueAck>, AppError> {
    Ok(Json(state.auth_use_cases.send_email_code(payload).await?))
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
    let profile = fetch_profile(&state.pool, user.id).await?;
    let role_profile = fetch_role_profile(&state.pool, &user).await?;
    let verification = fetch_latest_verification(&state.pool, user.id).await?;
    Ok(Json(AuthMeResponse {
        user: UserPublicView::from(user),
        profile,
        role_profile,
        verification,
    }))
}

pub async fn select_onboarding_role(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<SelectRoleInput>,
) -> Result<Json<AuthMeResponse>, AppError> {
    if user.role != UserRole::Unassigned {
        return Err(AppError::forbidden("role has already been assigned"));
    }
    if !matches!(payload.role, UserRole::Seeker | UserRole::Agent | UserRole::Landlord) {
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

    Ok(Json(AuthMeResponse {
        user: UserPublicView::from(updated),
        profile,
        role_profile,
        verification,
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
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(user.id)
    .bind(payload.phone.as_ref().map(|value| value.trim().to_string()))
    .bind(payload.bio.as_ref().map(|value| value.trim().to_string()))
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

    Ok(Json(AuthMeResponse {
        user: UserPublicView::from(refreshed_user),
        profile,
        role_profile,
        verification,
    }))
}

pub async fn create_verification(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateVerificationInput>,
) -> Result<(StatusCode, Json<VerificationView>), AppError> {
    if !matches!(user.role, UserRole::Agent | UserRole::Landlord) {
        return Err(AppError::forbidden("only agent and landlord accounts can submit verification"));
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
            p.status,
            p.description,
            COUNT(r.id)::bigint AS response_count,
            p.created_at
        FROM posts p
        INNER JOIN users u ON u.id = p.author_id
        LEFT JOIN responses r ON r.post_id = p.id
        WHERE p.author_id = $1
        GROUP BY p.id, u.full_name, u.role
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
        return Err(AppError::forbidden("only agents and landlords can send offers"));
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
        return Err(AppError::forbidden("you can only send offers with properties you own or manage"));
    }

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

    Ok((StatusCode::CREATED, Json(offer)))
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
            o.accepted_at, o.declined_at, o.created_at, o.updated_at
        FROM offers o
        INNER JOIN posts p ON p.id = o.need_post_id
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
        return Err(AppError::forbidden("only seekers can remove saved properties"));
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
) -> Result<Json<Vec<SavedPropertyView>>, AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden("only seekers can view saved properties"));
    }
    let items = sqlx::query_as::<_, SavedPropertyView>(
        "SELECT id, user_id, property_id, created_at FROM saved_properties WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user.id)
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

    let provider_user_id = sqlx::query_scalar::<_, Uuid>("SELECT provider_user_id FROM offers WHERE id = $1")
        .bind(payload.offer_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("offer not found"))?;

    let booking = sqlx::query_as::<_, BookingView>(
        r#"
        INSERT INTO bookings (id, offer_id, property_id, seeker_user_id, provider_user_id, booking_type, scheduled_for, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, offer_id, property_id, unit_id, seeker_user_id, provider_user_id,
                  booking_type, scheduled_for, status, notes, created_at, updated_at
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

    Ok((StatusCode::CREATED, Json(booking)))
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
        SELECT id, offer_id, property_id, unit_id, seeker_user_id, provider_user_id,
               booking_type, scheduled_for, status, notes, created_at, updated_at
        FROM bookings
        WHERE seeker_user_id = $1
        ORDER BY created_at DESC
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
        SELECT id, offer_id, property_id, unit_id, seeker_user_id, provider_user_id,
               booking_type, scheduled_for, status, notes, created_at, updated_at
        FROM bookings
        WHERE provider_user_id = $1
        ORDER BY created_at DESC
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
        return Err(AppError::forbidden("only landlords can view landlord properties"));
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
    let total_properties =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM properties").fetch_one(&state.pool).await?;
    let active_users = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE is_banned = FALSE",
    )
    .fetch_one(&state.pool)
    .await?;
    let monthly_revenue = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT COALESCE(SUM(amount), 0)::bigint FROM transactions WHERE status = 'succeeded' AND created_at >= date_trunc('month', NOW())",
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(0);
    let open_disputes = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM disputes WHERE status IN ('open', 'in_review', 'escalated')",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(AdminOverviewMetrics {
        total_properties,
        active_users,
        monthly_revenue,
        open_disputes,
    }))
}

pub async fn admin_list_verifications(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<AdminVerificationQueueItem>>, AppError> {
    ensure_admin(&user)?;
    let items = sqlx::query_as::<_, AdminVerificationQueueItem>(
        r#"
        SELECT
            v.id,
            v.user_id,
            u.email AS user_email,
            u.role::text AS user_role,
            v.status,
            v.submitted_at,
            v.reviewed_at,
            v.rejection_reason,
            v.notes,
            v.created_at,
            v.updated_at
        FROM verifications v
        INNER JOIN users u ON u.id = v.user_id
        ORDER BY v.created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(items))
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
    if !matches!(mapped_status, "submitted" | "in_review" | "approved" | "rejected" | "expired") {
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
    );
    state.mail_service.send(email).await?;

    Ok(Json(verification))
}

pub async fn seeker_dashboard_overview(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role != UserRole::Seeker {
        return Err(AppError::forbidden("only seekers can view seeker dashboard"));
    }
    let need_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts WHERE author_id = $1")
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
    let top_listings = list_agent_properties(State(state.clone()), AuthUser(user.clone())).await?.0;
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
    let property_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM properties WHERE owner_id = $1",
    )
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
    let maintenance_queue = list_landlord_maintenance(State(state.clone()), AuthUser(user.clone())).await?.0;

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
        return Err(AppError::forbidden("only agents can update agent properties"));
    }
    let updated = sqlx::query(
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
            updated_at = NOW()
        WHERE id = $1 AND (owner_id = $2 OR agent_id = $2)
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(payload.title)
    .bind(payload.description)
    .bind(payload.price)
    .bind(payload.location)
    .bind(payload.exact_address)
    .bind(payload.images)
    .bind(payload.contact_name)
    .bind(payload.contact_phone)
    .execute(&state.pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::not_found("property not found"));
    }
    let detail = state.property_use_cases.get_by_id(id, Some(&user)).await?;
    Ok(Json(json!(detail)))
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
    let file_key = format!("{}/{}-{}", payload.category, Uuid::new_v4(), payload.filename);
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
) -> Result<Json<Value>, AppError> {
    ensure_admin(&user)?;
    let items = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(to_jsonb(x)), '[]'::jsonb)
        FROM (
            SELECT id, full_name, email, role::text AS role, email_verified, verification_status, is_banned, created_at
            FROM users
            ORDER BY created_at DESC
        ) x
        "#,
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_admin_properties(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    ensure_admin(&user)?;
    let items = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(to_jsonb(x)), '[]'::jsonb)
        FROM (
            SELECT id, owner_id, agent_id, title, location, price, status::text AS status, created_at
            FROM properties
            ORDER BY created_at DESC
        ) x
        "#,
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_admin_transactions(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    ensure_admin(&user)?;
    let items = sqlx::query_scalar::<_, Value>(
        "SELECT COALESCE(jsonb_agg(to_jsonb(t)), '[]'::jsonb) FROM (SELECT * FROM transactions ORDER BY created_at DESC) t",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_admin_disputes(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    ensure_admin(&user)?;
    let items = sqlx::query_scalar::<_, Value>(
        "SELECT COALESCE(jsonb_agg(to_jsonb(d)), '[]'::jsonb) FROM (SELECT * FROM disputes ORDER BY created_at DESC) d",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_admin_reports(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    ensure_admin(&user)?;
    let items = sqlx::query_scalar::<_, Value>(
        "SELECT COALESCE(jsonb_agg(to_jsonb(r)), '[]'::jsonb) FROM (SELECT * FROM reports ORDER BY created_at DESC) r",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(items))
}

pub async fn list_admin_announcements(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    ensure_admin(&user)?;
    let items = sqlx::query_scalar::<_, Value>(
        "SELECT COALESCE(jsonb_agg(to_jsonb(a)), '[]'::jsonb) FROM (SELECT * FROM announcements ORDER BY created_at DESC) a",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(items))
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
    .bind(payload.title)
    .bind(payload.body)
    .bind(payload.audience)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn list_notifications(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Value>, AppError> {
    let items = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT COALESCE(jsonb_agg(to_jsonb(x)), '[]'::jsonb)
        FROM (
            SELECT id, type AS kind, title, body, data_json->>'actionUrl' AS "actionUrl", read_at AS "readAt", created_at AS "createdAt"
            FROM notifications
            WHERE user_id = $1
            ORDER BY created_at DESC
        ) x
        "#,
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(items))
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
        SELECT id, user_id, full_name, phone, city, avatar_url, bio, onboarding_completed, created_at, updated_at
        FROM profiles
        WHERE user_id = $1
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
        UserRole::Seeker => sqlx::query_scalar::<_, Value>(
            "SELECT to_jsonb(sp) FROM seeker_profiles sp WHERE sp.user_id = $1",
        )
        .bind(user.id)
        .fetch_optional(pool)
        .await?,
        UserRole::Agent => sqlx::query_scalar::<_, Value>(
            "SELECT to_jsonb(ap) FROM agent_profiles ap WHERE ap.user_id = $1",
        )
        .bind(user.id)
        .fetch_optional(pool)
        .await?,
        UserRole::Landlord => sqlx::query_scalar::<_, Value>(
            "SELECT to_jsonb(lp) FROM landlord_profiles lp WHERE lp.user_id = $1",
        )
        .bind(user.id)
        .fetch_optional(pool)
        .await?,
        UserRole::Admin => None,
    };
    Ok(value)
}

async fn fetch_latest_verification(pool: &PgPool, user_id: Uuid) -> Result<Option<VerificationView>, AppError> {
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
        return Err(AppError::forbidden("only landlords can access this endpoint"));
    }
    Ok(())
}

fn ensure_admin(user: &User) -> Result<(), AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::forbidden("only admins can access this endpoint"));
    }
    Ok(())
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
