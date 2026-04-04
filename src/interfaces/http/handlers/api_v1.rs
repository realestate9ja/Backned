use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    domain::{
        posts::{CreatePostInput, PostListItem, PostQuery},
        properties::{PropertyListItem, PropertyQuery},
        users::{AuthResponse, UpdateAgentVerificationInput, User, UserPublicView, UserRole},
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
#[serde(rename_all = "camelCase")]
pub struct OnboardingProfileInput {
    pub role: UserRole,
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
    if payload.role != user.role {
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
        UserRole::Admin => {}
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
