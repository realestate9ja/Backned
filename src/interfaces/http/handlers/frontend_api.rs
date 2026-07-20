use std::collections::HashMap;

use axum::{
    Json,
    body::Bytes,
    extract::{OriginalUri, Path, Query, State},
    http::Method,
    response::Redirect,
};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::infrastructure::email::service::{header_asset_url, HEADER_SECURITY_DARK};

use crate::{
    domain::users::{LoginInput, RegisterUserInput, User, UserRole, VerifyEmailInput, 
                    SendPasswordResetInput, ResetPasswordInput},
    infrastructure::auth::PasswordService,
    interfaces::http::{errors::AppError, middleware::auth::OptionalAuthUser, state::AppState},
};

const PUBLIC_APP_BASE_URL: &str = "https://verinest.up.railway.app";

fn ok(data: Value) -> Json<Value> {
    Json(json!({"status": "success", "data": data}))
}

fn top(data: Value) -> Json<Value> {
    Json(data)
}

fn frontend_role(role: UserRole) -> &'static str {
    match role {
        UserRole::Unassigned | UserRole::Seeker => "user",
        UserRole::Agent => "worker",
        UserRole::Landlord => "employer",
        UserRole::Admin | UserRole::SuperAdmin => "admin",
    }
}

fn backend_role(value: &str) -> Option<UserRole> {
    match value.trim().to_lowercase().as_str() {
        "user" | "unassigned" | "seeker" => Some(UserRole::Unassigned),
        "worker" | "agent" => Some(UserRole::Agent),
        "employer" | "landlord" => Some(UserRole::Landlord),
        "admin" => Some(UserRole::Admin),
        _ => None,
    }
}

fn parse_body(body: &Bytes) -> Result<Value, AppError> {
    if body.is_empty() {
        Ok(json!({}))
    } else {
        serde_json::from_slice(body).map_err(|_| AppError::bad_request("invalid json body"))
    }
}

fn str_field(body: &Value, key: &str) -> Result<String, AppError> {
    body.get(key)
        .and_then(Value::as_str)
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::bad_request(format!("{key} is required")))
}

fn opt_str_field(body: &Value, key: &str) -> Option<String> {
    body.get(key)
        .and_then(Value::as_str)
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn i64_field(body: &Value, key: &str) -> Result<i64, AppError> {
    if let Some(v) = body.get(key) {
        if let Some(i) = v.as_i64() {
            return Ok(i);
        }
        if let Some(f) = v.as_f64() {
            return Ok(f.round() as i64);
        }
        if let Some(s) = v.as_str() {
            return s
                .trim()
                .parse::<f64>()
                .map(|v| v.round() as i64)
                .map_err(|_| AppError::bad_request(format!("{key} must be numeric")));
        }
    }
    Err(AppError::bad_request(format!("{key} is required")))
}

fn i32_field(body: &Value, key: &str) -> Result<i32, AppError> {
    i64_field(body, key).and_then(|v| {
        i32::try_from(v).map_err(|_| AppError::bad_request(format!("{key} is out of range")))
    })
}

fn bool_field(body: &Value, key: &str, default: bool) -> bool {
    body.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn uuid_field(body: &Value, key: &str) -> Result<Uuid, AppError> {
    let value = str_field(body, key)?;
    Uuid::parse_str(&value)
        .map_err(|_| AppError::bad_request(format!("{key} must be a valid uuid")))
}

fn require_user(user: Option<User>) -> Result<User, AppError> {
    user.ok_or_else(|| AppError::unauthorized("authentication required"))
}

fn require_admin(user: &User) -> Result<(), AppError> {
    if user.role != UserRole::Admin {
        return Err(AppError::forbidden("admin access required"));
    }
    Ok(())
}

fn require_transaction_pin(pin: &str) -> Result<(), AppError> {
    if pin.len() != 6 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::bad_request("pin must be exactly 6 digits"));
    }
    Ok(())
}

async fn frontend_user_json(pool: &PgPool, user_id: Uuid) -> Result<Value, AppError> {
    let row = sqlx::query(
        r#"
        SELECT u.id, u.full_name, u.email, u.email_verified, u.username, u.role,
               u.verification_status, u.document_verified, u.trust_score,
               u.wallet_address, u.bank_account_linked, u.profile_completed,
               u.transaction_pin_hash IS NOT NULL AS has_transaction_pin,
               u.phone, u.address, u.nationality, u.lga, u.dob,
               u.referral_code, u.referral_count, u.verification_type,
               u.verification_number, u.nearest_landmark, u.google_id,
               u.role_change_count, u.role_change_reset_at,
               p.avatar_url, p.city, p.bio
        FROM users u
        LEFT JOIN profiles p ON p.user_id = u.id
        WHERE u.id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let role: UserRole = row.try_get("role")?;
    let verification_status: String = row.try_get("verification_status")?;

    Ok(json!({
        "id": row.get::<Uuid, _>("id"),
        "name": row.get::<String, _>("full_name"),
        "email": row.get::<String, _>("email"),
        "username": row.try_get::<Option<String>, _>("username")?.unwrap_or_else(|| format!("user_{}", &user_id.to_string()[..8])),
        "email_verified": row.get::<bool, _>("email_verified"),
        "verified": row.get::<bool, _>("email_verified"),
        "role": frontend_role(role),
        "verification_status": verification_status.clone(),
        "document_verified": row.get::<bool, _>("document_verified"),
        "trust_score": row.get::<i32, _>("trust_score"),
        "wallet_created": row.try_get::<Option<String>, _>("wallet_address")?.is_some(),
        "wallet_address": row.try_get::<Option<String>, _>("wallet_address")?,
        "bank_account_linked": row.get::<bool, _>("bank_account_linked"),
        "profile_completed": row.get::<bool, _>("profile_completed"),
        "transaction_pin": if row.get::<bool, _>("has_transaction_pin") { 1 } else { 0 },
        "phone": row.try_get::<Option<String>, _>("phone")?,
        "address": row.try_get::<Option<String>, _>("address")?,
        "nationality": row.try_get::<Option<String>, _>("nationality")?,
        "lga": row.try_get::<Option<String>, _>("lga")?,
        "dob": row.try_get::<Option<DateTime<Utc>>, _>("dob")?,
        "avatar_url": row.try_get::<Option<String>, _>("avatar_url")?,
        "city": row.try_get::<Option<String>, _>("city")?,
        "bio": row.try_get::<Option<String>, _>("bio")?,
        "referral_code": row.try_get::<Option<String>, _>("referral_code")?,
        "referral_count": row.get::<i32, _>("referral_count"),
        "verification_type": row.try_get::<Option<String>, _>("verification_type")?,
        "verification_number": row.try_get::<Option<String>, _>("verification_number")?,
        "nearest_landmark": row.try_get::<Option<String>, _>("nearest_landmark")?,
        "google_id": row.try_get::<Option<String>, _>("google_id")?,
        "role_change_count": row.get::<i32, _>("role_change_count"),
        "role_change_reset_at": row.try_get::<Option<DateTime<Utc>>, _>("role_change_reset_at")?,
        "kyc_verified": match verification_status.as_str() {
            "approved" | "verified" => "verified",
            "submitted" | "pending" | "in_review" => "pending",
            "rejected" => "rejected",
            _ => "unverified"
        }
    }))
}

async fn send_verification_link(state: &AppState, user: &User) -> Result<(), AppError> {
    let token = state
        .user_repository
        .create_email_verification_token(user.id)
        .await?;
    let link = format!("{}/api/auth/verify?token={}", PUBLIC_APP_BASE_URL, token);
    let email = state
        .mail_service
        .verification_email(user.email.clone(), &user.full_name, &link, &header_asset_url(HEADER_SECURITY_DARK));
    state.mail_service.send(email).await.map_err(AppError::from)
}

async fn create_notification(
    pool: &PgPool,
    user_id: Uuid,
    kind: &str,
    title: &str,
    body: &str,
    data: Value,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO notifications (id, user_id, type, title, body, data_json) VALUES ($1, $2, $3, $4, $5, $6)")
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

async fn wallet_summary(pool: &PgPool, user_id: Uuid) -> Result<Value, AppError> {
    let balance = sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT COALESCE(SUM(CASE
            WHEN type IN ('charge', 'refund', 'escrow_release', 'rent_collection') AND status IN ('succeeded', 'completed') THEN amount
            WHEN type IN ('payout', 'fee', 'escrow_hold') AND status IN ('succeeded', 'completed', 'processing') THEN -amount
            ELSE 0 END), 0)
        FROM transactions
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let row = sqlx::query(
        r#"
        SELECT wallet_address,
               (SELECT account_number FROM wallet_bank_accounts WHERE user_id = $1 AND is_primary = TRUE ORDER BY created_at DESC LIMIT 1) AS account_number,
               (SELECT bank_code FROM wallet_bank_accounts WHERE user_id = $1 AND is_primary = TRUE ORDER BY created_at DESC LIMIT 1) AS bank_code
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(json!({
        "id": row.try_get::<Option<String>, _>("wallet_address")?.unwrap_or_default(),
        "balance": balance,
        "currency": "NGN",
        "wallet_created": row.try_get::<Option<String>, _>("wallet_address")?.is_some(),
        "account_number": row.try_get::<Option<String>, _>("account_number")?,
        "bank_name": row.try_get::<Option<String>, _>("bank_code")?
    }))
}

async fn list_notifications_response(
    pool: &PgPool,
    user_id: Uuid,
    page: i64,
    limit: i64,
) -> Result<Value, AppError> {
    let offset = (page - 1).max(0) * limit.max(1);
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, title, body, type, read_at, created_at
        FROM notifications
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let total =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notifications WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    let unread = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND read_at IS NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let notifications = rows.into_iter().map(|row| json!({
        "id": row.get::<Uuid, _>("id"),
        "user_id": row.get::<Uuid, _>("user_id"),
        "title": row.get::<String, _>("title"),
        "message": row.get::<String, _>("body"),
        "notification_type": row.get::<String, _>("type"),
        "is_read": row.try_get::<Option<DateTime<Utc>>, _>("read_at").ok().flatten().is_some(),
        "created_at": row.get::<DateTime<Utc>, _>("created_at")
    })).collect::<Vec<_>>();

    Ok(json!({
        "notifications": notifications,
        "total": total,
        "page": page,
        "limit": limit,
        "unread_count": unread
    }))
}

fn job_value(row: &sqlx::postgres::PgRow) -> Value {
    json!({
        "id": row.get::<Uuid, _>("id"),
        "title": row.get::<String, _>("title"),
        "description": row.get::<String, _>("description"),
        "category": row.get::<String, _>("category"),
        "budget": row.get::<i64, _>("budget"),
        "location_state": row.get::<String, _>("location_state"),
        "location_city": row.get::<String, _>("location_city"),
        "location_address": row.get::<String, _>("location_address"),
        "estimated_duration_days": row.get::<i32, _>("estimated_duration_days"),
        "partial_payment_allowed": row.get::<bool, _>("partial_payment_allowed"),
        "partial_payment_percentage": row.try_get::<Option<f64>, _>("partial_payment_percentage").ok().flatten(),
        "deadline": row.try_get::<Option<DateTime<Utc>>, _>("deadline").ok().flatten(),
        "status": row.get::<String, _>("status"),
        "created_at": row.get::<DateTime<Utc>, _>("created_at")
    })
}

async fn worker_profile_value(pool: &PgPool, worker_id: Uuid) -> Result<Value, AppError> {
    let row = sqlx::query(
        r#"
        SELECT wp.*, u.full_name, u.email, u.username, u.document_verified, u.trust_score, p.avatar_url
        FROM labour_worker_profiles wp
        JOIN users u ON u.id = wp.user_id
        LEFT JOIN profiles p ON p.user_id = u.id
        WHERE wp.user_id = $1 OR wp.id = $1
        LIMIT 1
        "#,
    )
    .bind(worker_id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Err(AppError::not_found("worker not found"));
    };
    let user_id: Uuid = row.get("user_id");
    let portfolio_rows = sqlx::query("SELECT id, title, description, image_url, project_date, created_at FROM labour_worker_portfolio WHERE user_id = $1 ORDER BY created_at DESC")
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    let reviews = sqlx::query(
        r#"
        SELECT r.id, r.rating, r.comment, r.created_at, u.full_name, u.username
        FROM reviews r JOIN users u ON u.id = r.reviewer_id
        WHERE r.reviewee_id = $1 ORDER BY r.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(json!({
        "id": row.get::<Uuid, _>("id"),
        "user_id": user_id,
        "category": row.get::<String, _>("category"),
        "experience_years": row.get::<i32, _>("experience_years"),
        "description": row.get::<String, _>("description"),
        "hourly_rate": row.get::<i64, _>("hourly_rate"),
        "daily_rate": row.get::<i64, _>("daily_rate"),
        "location_state": row.get::<String, _>("location_state"),
        "location_city": row.get::<String, _>("location_city"),
        "is_available": row.get::<bool, _>("is_available"),
        "skills": row.try_get::<Value, _>("skills").unwrap_or_else(|_| json!([])),
        "user": {
            "id": user_id,
            "name": row.get::<String, _>("full_name"),
            "username": row.try_get::<Option<String>, _>("username").ok().flatten(),
            "email": row.get::<String, _>("email"),
            "avatar_url": row.try_get::<Option<String>, _>("avatar_url").ok().flatten(),
            "trust_score": row.get::<i32, _>("trust_score"),
            "verified": row.get::<bool, _>("document_verified")
        },
        "portfolio": portfolio_rows.into_iter().map(|row| json!({
            "id": row.get::<Uuid, _>("id"),
            "title": row.get::<String, _>("title"),
            "description": row.get::<String, _>("description"),
            "image_url": row.get::<String, _>("image_url"),
            "project_date": row.get::<chrono::NaiveDate, _>("project_date"),
            "created_at": row.get::<DateTime<Utc>, _>("created_at")
        })).collect::<Vec<_>>(),
        "reviews": reviews.into_iter().map(|row| json!({
            "id": row.get::<Uuid, _>("id"),
            "rating": row.get::<i32, _>("rating"),
            "comment": row.get::<String, _>("comment"),
            "created_at": row.get::<DateTime<Utc>, _>("created_at"),
            "reviewer": {"name": row.get::<String, _>("full_name"), "username": row.try_get::<Option<String>, _>("username").ok().flatten()}
        })).collect::<Vec<_>>()
    }))
}

pub async fn oauth_google() -> Redirect {
    Redirect::temporary("/login")
}

pub async fn dispatch(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    maybe_user: OptionalAuthUser,
    method: Method,
    OriginalUri(uri): OriginalUri,
    body: Bytes,
) -> Result<Json<Value>, AppError> {
    let user = maybe_user.0;
    let body = parse_body(&body)?;
    let segments = path
        .split('/')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    match (method.as_str(), segments.as_slice()) {
        ("POST", ["auth", "register"]) => {
            let name = str_field(&body, "name")?;
            let username = str_field(&body, "username")?;
            let email = str_field(&body, "email")?;
            let password = str_field(&body, "password")?;
            let confirm = str_field(&body, "passwordConfirm")?;
            if password != confirm {
                return Err(AppError::bad_request(
                    "password confirmation does not match",
                ));
            }
            let count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE LOWER(username) = LOWER($1)",
            )
            .bind(&username)
            .fetch_one(&state.pool)
            .await?;
            if count > 0 {
                return Err(AppError::conflict("username already taken"));
            }
            let auth = state
                .auth_use_cases
                .register(RegisterUserInput {
                    full_name: name,
                    email,
                    password,
                    role: UserRole::Unassigned,
                    phone: None,
                    bio: None,
                })
                .await?;
            sqlx::query("UPDATE users SET username = $2, referral_code = COALESCE(referral_code, $3), role_change_reset_at = COALESCE(role_change_reset_at, NOW()) WHERE id = $1")
                .bind(auth.user.id)
                .bind(username)
                .bind(opt_str_field(&body, "referral_code").unwrap_or_else(|| format!("REF-{}", &auth.user.id.to_string()[..8])))
                .execute(&state.pool)
                .await?;
            let created = state
                .user_repository
                .find_by_id(auth.user.id)
                .await?
                .ok_or_else(|| AppError::not_found("user not found"))?;
            send_verification_link(&state, &created).await?;
            return Ok(top(
                json!({"status": "success", "token": auth.token, "refresh_token": auth.refresh_token, "user": frontend_user_json(&state.pool, created.id).await?}),
            ));
        }
        ("POST", ["auth", "login"]) => {
            let auth = state
                .auth_use_cases
                .login(LoginInput {
                    email: str_field(&body, "email")?,
                    password: str_field(&body, "password")?,
                })
                .await?;
            return Ok(top(
                json!({"status": "success", "token": auth.token, "refresh_token": auth.refresh_token, "user": frontend_user_json(&state.pool, auth.user.id).await?}),
            ));
        }
        ("GET", ["auth", "verify"]) => {
            let token = query
                .get("token")
                .cloned()
                .ok_or_else(|| AppError::bad_request("token is required"))?;
            let payload = VerifyEmailInput { token };
            let _ = payload;
            let token_value = query.get("token").cloned().unwrap();
            let found = state
                .user_repository
                .find_by_email_verification_token(&token_value)
                .await?
                .ok_or_else(|| AppError::bad_request("invalid or expired verification token"))?;
            let updated = state
                .user_repository
                .mark_email_verified(found.id)
                .await?
                .ok_or_else(|| AppError::not_found("user not found"))?;
            state
                .user_repository
                .mark_email_verification_token_used(&token_value)
                .await?;
            
            use crate::infrastructure::auth::CsrfService;
            let csrf_token = CsrfService::generate_token();
            let token = state.jwt_service.generate_token(&updated, &csrf_token)?;
            let refresh_token = state
                .user_repository
                .create_refresh_token(updated.id, Utc::now() + chrono::Duration::days(30))
                .await?;
            return Ok(top(
                json!({"status": "success", "token": token, "refresh_token": refresh_token, "csrf_token": csrf_token, "user": frontend_user_json(&state.pool, updated.id).await?}),
            ));
        }
        ("POST", ["auth", "resend-verification"]) => {
            let email = str_field(&body, "email")?;
            let found = state
                .user_repository
                .find_by_email(&email)
                .await?
                .ok_or_else(|| AppError::not_found("user not found"))?;
            send_verification_link(&state, &found).await?;
            return Ok(ok(json!({"sent": true})));
        }
        ("POST", ["auth", "send-transaction-otp"]) => {
            let user = require_user(user)?;
            let code = state
                .user_repository
                .create_email_verification_code(user.id, &user.email, "transaction_otp")
                .await?;
            let email = state.mail_service.verification_code_email(
                user.email.clone(),
                &user.full_name,
                &code,
                &header_asset_url(HEADER_SECURITY_DARK),
            );
            state.mail_service.send(email).await?;
            return Ok(ok(json!({"sent": true, "code_length": 5})));
        }
        ("POST", ["auth", "verify-transaction-otp"]) => {
            let user = require_user(user)?;
            let otp = str_field(&body, "otp")?;
            let found = state
                .user_repository
                .find_by_email_verification_code(&user.email, &otp)
                .await?;
            if found.is_some() {
                state
                    .user_repository
                    .mark_email_verification_code_used(&user.email, &otp)
                    .await?;
                return Ok(top(json!({"verified": true, "message": "OTP verified"})));
            }
            return Ok(top(
                json!({"verified": false, "message": "Invalid OTP. Please try again."}),
            ));
        }
        ("POST", ["auth", "reset-transaction-pin"]) => {
            let user = require_user(user)?;
            let code = state
                .user_repository
                .create_email_verification_code(user.id, &user.email, "transaction_pin_reset")
                .await?;
            let email = state.mail_service.verification_code_email(
                user.email.clone(),
                &user.full_name,
                &code,
                &header_asset_url(HEADER_SECURITY_DARK),
            );
            state.mail_service.send(email).await?;
            return Ok(ok(json!({"sent": true, "code_length": 5})));
        }
        ("POST", ["auth", "send-password-reset"]) => {
            let _result = state
                .auth_use_cases
                .send_password_reset(SendPasswordResetInput {
                    email: str_field(&body, "email")?,
                })
                .await?;
            return Ok(ok(json!({
                "ok": true,
                "message": "Password reset email sent. Check your inbox for the link."
            })));
        }
        ("POST", ["auth", "reset-password"]) => {
            let result = state
                .auth_use_cases
                .reset_password(ResetPasswordInput {
                    token: str_field(&body, "token")?,
                    password: str_field(&body, "password")?,
                })
                .await?;
            return Ok(top(
                json!({"status": "success", "user": frontend_user_json(&state.pool, result.id).await?}),
            ));
        }
        ("GET", ["users", "me"]) => {
            let user = require_user(user)?;
            return Ok(ok(
                json!({"user": frontend_user_json(&state.pool, user.id).await?}),
            ));
        }
        ("PUT", ["users", "role"]) => {
            let user = require_user(user)?;
            let role = backend_role(&str_field(&body, "role")?)
                .ok_or_else(|| AppError::bad_request("invalid role"))?;
            let updated = state
                .user_repository
                .update_role(user.id, role)
                .await?
                .ok_or_else(|| AppError::not_found("user not found"))?;
            sqlx::query("UPDATE users SET role_change_count = role_change_count + 1, role_change_reset_at = COALESCE(role_change_reset_at, NOW()) WHERE id = $1")
                .bind(updated.id)
                .execute(&state.pool)
                .await?;
            return Ok(top(
                json!({"user": frontend_user_json(&state.pool, updated.id).await?}),
            ));
        }
        ("PUT", ["users", "role", "upgrade"]) => {
            let actor = require_user(user)?;
            let target = opt_str_field(&body, "target_user_id")
                .map(|v| {
                    Uuid::parse_str(&v)
                        .map_err(|_| AppError::bad_request("target_user_id must be a valid uuid"))
                })
                .transpose()?
                .unwrap_or(actor.id);
            if target != actor.id && actor.role != UserRole::Admin {
                return Err(AppError::forbidden("cannot upgrade another user"));
            }
            let new_role_raw =
                opt_str_field(&body, "new_role").unwrap_or_else(|| "user".to_string());
            let role =
                backend_role(&new_role_raw).ok_or_else(|| AppError::bad_request("invalid role"))?;
            let updated = state
                .user_repository
                .update_role(target, role)
                .await?
                .ok_or_else(|| AppError::not_found("user not found"))?;
            return Ok(ok(
                json!({"user": frontend_user_json(&state.pool, updated.id).await?}),
            ));
        }
        ("PUT", ["users", "name"]) => {
            let user = require_user(user)?;
            let name = str_field(&body, "name")?;
            sqlx::query("UPDATE users SET full_name = $2, updated_at = NOW() WHERE id = $1")
                .bind(user.id)
                .bind(&name)
                .execute(&state.pool)
                .await?;
            sqlx::query(
                "UPDATE profiles SET full_name = $2, updated_at = NOW() WHERE user_id = $1",
            )
            .bind(user.id)
            .bind(&name)
            .execute(&state.pool)
            .await?;
            return Ok(ok(
                json!({"user": frontend_user_json(&state.pool, user.id).await?}),
            ));
        }
        ("GET", ["users", "check-username"]) => {
            let username = query
                .get("username")
                .cloned()
                .ok_or_else(|| AppError::bad_request("username is required"))?;
            let count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE LOWER(username) = LOWER($1)",
            )
            .bind(username)
            .fetch_one(&state.pool)
            .await?;
            return Ok(top(json!({"available": count == 0})));
        }
        ("PUT", ["users", "username"]) => {
            let user = require_user(user)?;
            let username = str_field(&body, "username")?;
            let count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE LOWER(username) = LOWER($1) AND id <> $2",
            )
            .bind(&username)
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
            if count > 0 {
                return Err(AppError::conflict("username already taken"));
            }
            sqlx::query("UPDATE users SET username = $2, updated_at = NOW() WHERE id = $1")
                .bind(user.id)
                .bind(username)
                .execute(&state.pool)
                .await?;
            return Ok(ok(
                json!({"user": frontend_user_json(&state.pool, user.id).await?}),
            ));
        }
        ("PUT", ["users", "password"]) => {
            let user = require_user(user)?;
            let old_password = str_field(&body, "old_password")?;
            let new_password = str_field(&body, "new_password")?;
            let new_confirm = str_field(&body, "new_password_confirm")?;
            if new_password != new_confirm {
                return Err(AppError::bad_request(
                    "password confirmation does not match",
                ));
            }
            let password_service = PasswordService;
            if !password_service.verify_password(&old_password, &user.password_hash)? {
                return Err(AppError::unauthorized("invalid password"));
            }
            let hash = password_service.hash_password(&new_password)?;
            sqlx::query("UPDATE users SET password_hash = $2, updated_at = NOW() WHERE id = $1")
                .bind(user.id)
                .bind(hash)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"updated": true})));
        }
        ("POST", ["users", "verify-password"]) => {
            let user = require_user(user)?;
            let password_service = PasswordService;
            let verified = password_service
                .verify_password(&str_field(&body, "password")?, &user.password_hash)?;
            return Ok(top(
                json!({"status": if verified { "success" } else { "error" }, "verified": verified}),
            ));
        }
        ("PUT", ["users", "transaction-pin"]) => {
            let user = require_user(user)?;
            let new_pin = str_field(&body, "new_pin")?;
            require_transaction_pin(&new_pin)?;
            let password_service = PasswordService;
            if let Some(password) = opt_str_field(&body, "password") {
                if !password_service.verify_password(&password, &user.password_hash)? {
                    return Err(AppError::unauthorized("invalid password"));
                }
            }
            if let Some(current_pin) = opt_str_field(&body, "current_pin") {
                require_transaction_pin(&current_pin)?;
                let existing = sqlx::query_scalar::<_, Option<String>>(
                    "SELECT transaction_pin_hash FROM users WHERE id = $1",
                )
                .bind(user.id)
                .fetch_one(&state.pool)
                .await?;
                let Some(existing) = existing else {
                    return Err(AppError::bad_request("transaction pin not set"));
                };
                if !password_service.verify_password(&current_pin, &existing)? {
                    return Err(AppError::unauthorized("invalid current pin"));
                }
            }
            let hash = password_service.hash_password(&new_pin)?;
            sqlx::query(
                "UPDATE users SET transaction_pin_hash = $2, updated_at = NOW() WHERE id = $1",
            )
            .bind(user.id)
            .bind(hash)
            .execute(&state.pool)
            .await?;
            return Ok(ok(json!({"updated": true})));
        }
        ("POST", ["users", "transaction-pin", "verify"]) => {
            let user = require_user(user)?;
            let pin = str_field(&body, "transaction_pin")?;
            require_transaction_pin(&pin)?;
            let existing = sqlx::query_scalar::<_, Option<String>>(
                "SELECT transaction_pin_hash FROM users WHERE id = $1",
            )
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
            let verified = if let Some(existing) = existing {
                PasswordService.verify_password(&pin, &existing)?
            } else {
                false
            };
            return Ok(top(
                json!({"verified": verified, "message": if verified { "PIN verified" } else { "Invalid PIN. Please try again." }}),
            ));
        }
        ("GET", ["users", "avatar"]) => {
            let user = require_user(user)?;
            let avatar = sqlx::query_scalar::<_, Option<String>>(
                "SELECT avatar_url FROM profiles WHERE user_id = $1",
            )
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
            return Ok(ok(json!({"avatar_url": avatar})));
        }
        ("POST", ["users", "avatar"]) => {
            let user = require_user(user)?;
            let avatar_url = str_field(&body, "avatar_url")?;
            sqlx::query(
                "UPDATE profiles SET avatar_url = $2, updated_at = NOW() WHERE user_id = $1",
            )
            .bind(user.id)
            .bind(avatar_url)
            .execute(&state.pool)
            .await?;
            return Ok(ok(
                json!({"user": frontend_user_json(&state.pool, user.id).await?}),
            ));
        }
        ("POST", ["verification", "document"]) | ("POST", ["verification", "nin"]) => {
            let user = require_user(user)?;
            let document_id =
                str_field(&body, "document_id").or_else(|_| str_field(&body, "documentId"))?;
            let document_url =
                str_field(&body, "document_url").or_else(|_| str_field(&body, "documentUrl"))?;
            let document_type = opt_str_field(&body, "verification_type")
                .or_else(|| opt_str_field(&body, "documentType"))
                .or_else(|| opt_str_field(&body, "document_type"))
                .unwrap_or_else(|| "document".to_string());
            let verification_id = sqlx::query_scalar::<_, Option<Uuid>>(
                "SELECT id FROM verifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
            )
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?
            .unwrap_or(Uuid::new_v4());
            sqlx::query("INSERT INTO verifications (id, user_id, status, submitted_at, created_at, updated_at) VALUES ($1, $2, 'submitted', NOW(), NOW(), NOW()) ON CONFLICT (id) DO UPDATE SET status = 'submitted', submitted_at = NOW(), updated_at = NOW()")
                .bind(verification_id)
                .bind(user.id)
                .execute(&state.pool)
                .await?;
            sqlx::query("INSERT INTO verification_documents (id, verification_id, document_type, file_url, file_key, mime_type, status) VALUES ($1, $2, $3, $4, $5, $6, 'uploaded')")
                .bind(Uuid::new_v4())
                .bind(verification_id)
                .bind(document_type.clone())
                .bind(document_url.clone())
                .bind(document_url.clone())
                .bind("application/octet-stream")
                .execute(&state.pool)
                .await?;
            sqlx::query("UPDATE users SET verification_status = 'submitted', document_verified = FALSE, verification_type = $2, verification_number = $3, nationality = $4, lga = $5, nearest_landmark = $6, updated_at = NOW() WHERE id = $1")
                .bind(user.id)
                .bind(document_type)
                .bind(document_id)
                .bind(opt_str_field(&body, "nationality"))
                .bind(opt_str_field(&body, "lga"))
                .bind(opt_str_field(&body, "nearest_landmark").or_else(|| opt_str_field(&body, "nearestLandmark")))
                .execute(&state.pool)
                .await?;
            create_notification(
                &state.pool,
                user.id,
                "verification_submitted",
                "Verification submitted",
                "Your verification has been submitted for review.",
                json!({}),
            )
            .await?;
            return Ok(ok(json!({"verification_status": "submitted"})));
        }
        ("GET", ["verification", "complete-status"]) => {
            let user = require_user(user)?;
            let row = sqlx::query("SELECT status, submitted_at, reviewed_at, rejection_reason FROM verifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1")
                .bind(user.id)
                .fetch_optional(&state.pool)
                .await?;
            let data = row.map(|row| json!({
                "status": row.get::<String, _>("status"),
                "submitted_at": row.try_get::<Option<DateTime<Utc>>, _>("submitted_at").ok().flatten(),
                "reviewed_at": row.try_get::<Option<DateTime<Utc>>, _>("reviewed_at").ok().flatten(),
                "rejection_reason": row.try_get::<Option<String>, _>("rejection_reason").ok().flatten()
            })).unwrap_or_else(|| json!({"status": "not_started"}));
            return Ok(ok(data));
        }
        ("GET", ["verification", "admin", "pending"]) => {
            let user = require_user(user)?;
            require_admin(&user)?;
            let rows = sqlx::query("SELECT v.id, v.user_id, v.status, v.submitted_at, u.full_name, u.email FROM verifications v JOIN users u ON u.id = v.user_id WHERE v.status IN ('submitted', 'in_review') ORDER BY v.created_at DESC")
                .fetch_all(&state.pool)
                .await?;
            let data = rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "user_id": row.get::<Uuid, _>("user_id"), "status": row.get::<String, _>("status"), "submitted_at": row.try_get::<Option<DateTime<Utc>>, _>("submitted_at").ok().flatten(), "user": {"name": row.get::<String, _>("full_name"), "email": row.get::<String, _>("email")}})).collect::<Vec<_>>();
            return Ok(ok(json!(data)));
        }
        ("POST", ["verification", "admin", id, "review"]) => {
            let user = require_user(user)?;
            require_admin(&user)?;
            let verification_id = Uuid::parse_str(id)
                .map_err(|_| AppError::bad_request("invalid verification id"))?;
            let status = str_field(&body, "status")?.to_lowercase();
            sqlx::query("UPDATE verifications SET status = $2, reviewed_at = NOW(), reviewed_by = $3, notes = $4, rejection_reason = $5, updated_at = NOW() WHERE id = $1")
                .bind(verification_id)
                .bind(&status)
                .bind(user.id)
                .bind(opt_str_field(&body, "notes"))
                .bind(opt_str_field(&body, "rejection_reason"))
                .execute(&state.pool)
                .await?;
            let owner_id =
                sqlx::query_scalar::<_, Uuid>("SELECT user_id FROM verifications WHERE id = $1")
                    .bind(verification_id)
                    .fetch_one(&state.pool)
                    .await?;
            sqlx::query("UPDATE users SET verification_status = $2, document_verified = $3, updated_at = NOW() WHERE id = $1")
                .bind(owner_id)
                .bind(&status)
                .bind(matches!(status.as_str(), "approved" | "verified"))
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"updated": true})));
        }
        ("GET", ["labour", "worker", "dashboard"]) => {
            let user = require_user(user)?;
            let pending = sqlx::query("SELECT a.id, a.job_id, a.proposed_rate, a.estimated_completion, a.cover_letter, a.status, j.title FROM labour_job_applications a JOIN labour_jobs j ON j.id = a.job_id WHERE a.worker_user_id = $1 ORDER BY a.created_at DESC")
                .bind(user.id)
                .fetch_all(&state.pool)
                .await?;
            let active = sqlx::query("SELECT id, job_id, agreed_rate, agreed_timeline, status FROM labour_contracts WHERE worker_user_id = $1 AND status = 'active' ORDER BY created_at DESC")
                .bind(user.id)
                .fetch_all(&state.pool)
                .await?;
            let completed = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM labour_contracts WHERE worker_user_id = $1 AND status = 'completed'")
                .bind(user.id)
                .fetch_one(&state.pool)
                .await?;
            let earnings = sqlx::query_scalar::<_, Option<i64>>("SELECT COALESCE(SUM(agreed_rate),0) FROM labour_contracts WHERE worker_user_id = $1 AND status IN ('active', 'completed')")
                .bind(user.id)
                .fetch_one(&state.pool)
                .await?
                .unwrap_or(0);
            return Ok(ok(json!({
                "pending_applications": pending.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "job_id": row.get::<Uuid, _>("job_id"), "proposed_rate": row.get::<i64, _>("proposed_rate"), "estimated_completion": row.get::<i32, _>("estimated_completion"), "cover_letter": row.get::<String, _>("cover_letter"), "status": row.get::<String, _>("status"), "job": {"title": row.get::<String, _>("title")}})).collect::<Vec<_>>(),
                "active_contracts": active.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "job_id": row.try_get::<Option<Uuid>, _>("job_id").ok().flatten(), "agreed_rate": row.get::<i64, _>("agreed_rate"), "agreed_timeline": row.get::<i32, _>("agreed_timeline"), "status": row.get::<String, _>("status")})).collect::<Vec<_>>(),
                "active_jobs": [],
                "completed_jobs": completed,
                "total_earnings": earnings
            })));
        }
        ("GET", ["labour", "employer", "dashboard"]) => {
            let user = require_user(user)?;
            let jobs = sqlx::query(
                "SELECT * FROM labour_jobs WHERE employer_user_id = $1 ORDER BY created_at DESC",
            )
            .bind(user.id)
            .fetch_all(&state.pool)
            .await?;
            let contracts = sqlx::query("SELECT id, job_id, agreed_rate, agreed_timeline, status, worker_user_id FROM labour_contracts WHERE employer_user_id = $1 AND status = 'active' ORDER BY created_at DESC")
                .bind(user.id)
                .fetch_all(&state.pool)
                .await?;
            let completed = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM labour_jobs WHERE employer_user_id = $1 AND status = 'completed'")
                .bind(user.id)
                .fetch_one(&state.pool)
                .await?;
            let spent = sqlx::query_scalar::<_, Option<i64>>("SELECT COALESCE(SUM(agreed_rate),0) FROM labour_contracts WHERE employer_user_id = $1")
                .bind(user.id)
                .fetch_one(&state.pool)
                .await?
                .unwrap_or(0);
            return Ok(ok(json!({
                "posted_jobs": jobs.into_iter().map(|row| job_value(&row)).collect::<Vec<_>>(),
                "active_contracts": contracts.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "job_id": row.try_get::<Option<Uuid>, _>("job_id").ok().flatten(), "agreed_rate": row.get::<i64, _>("agreed_rate"), "agreed_timeline": row.get::<i32, _>("agreed_timeline"), "status": row.get::<String, _>("status"), "worker": {"id": row.get::<Uuid, _>("worker_user_id")}})).collect::<Vec<_>>(),
                "completed_jobs": completed,
                "total_spent": spent
            })));
        }
        ("GET", ["labour", "worker", "profile"]) => {
            let user = require_user(user)?;
            return Ok(ok(worker_profile_value(&state.pool, user.id).await?));
        }
        ("POST", ["labour", "worker", "profile"]) => {
            let user = require_user(user)?;
            let profile_id = sqlx::query_scalar::<_, Option<Uuid>>(
                "SELECT id FROM labour_worker_profiles WHERE user_id = $1",
            )
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?
            .unwrap_or(Uuid::new_v4());
            sqlx::query("INSERT INTO labour_worker_profiles (id, user_id, category, experience_years, description, hourly_rate, daily_rate, location_state, location_city, skills, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW()) ON CONFLICT (user_id) DO UPDATE SET category = EXCLUDED.category, experience_years = EXCLUDED.experience_years, description = EXCLUDED.description, hourly_rate = EXCLUDED.hourly_rate, daily_rate = EXCLUDED.daily_rate, location_state = EXCLUDED.location_state, location_city = EXCLUDED.location_city, skills = EXCLUDED.skills, updated_at = NOW()")
                .bind(profile_id)
                .bind(user.id)
                .bind(str_field(&body, "category")?)
                .bind(i32_field(&body, "experience_years")?)
                .bind(str_field(&body, "description")?)
                .bind(i64_field(&body, "hourly_rate")?)
                .bind(i64_field(&body, "daily_rate")?)
                .bind(str_field(&body, "location_state")?)
                .bind(str_field(&body, "location_city")?)
                .bind(body.get("skills").cloned().unwrap_or_else(|| json!([])))
                .execute(&state.pool)
                .await?;
            sqlx::query(
                "UPDATE users SET profile_completed = TRUE, updated_at = NOW() WHERE id = $1",
            )
            .bind(user.id)
            .execute(&state.pool)
            .await?;
            return Ok(ok(worker_profile_value(&state.pool, user.id).await?));
        }
        ("GET", ["labour", "worker", "portfolio"]) => {
            let user = require_user(user)?;
            let rows = sqlx::query("SELECT id, title, description, image_url, project_date, created_at FROM labour_worker_portfolio WHERE user_id = $1 ORDER BY created_at DESC")
                .bind(user.id)
                .fetch_all(&state.pool)
                .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "title": row.get::<String, _>("title"), "description": row.get::<String, _>("description"), "image_url": row.get::<String, _>("image_url"), "project_date": row.get::<chrono::NaiveDate, _>("project_date"), "created_at": row.get::<DateTime<Utc>, _>("created_at")})).collect::<Vec<_>>())));
        }
        ("POST", ["labour", "worker", "portfolio"]) => {
            let user = require_user(user)?;
            let project_date =
                chrono::NaiveDate::parse_from_str(&str_field(&body, "project_date")?, "%Y-%m-%d")
                    .map_err(|_| AppError::bad_request("project_date must be YYYY-MM-DD"))?;
            sqlx::query("INSERT INTO labour_worker_portfolio (id, user_id, title, description, image_url, project_date) VALUES ($1, $2, $3, $4, $5, $6)")
                .bind(Uuid::new_v4())
                .bind(user.id)
                .bind(str_field(&body, "title")?)
                .bind(str_field(&body, "description")?)
                .bind(str_field(&body, "image_url")?)
                .bind(project_date)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"created": true})));
        }
        ("DELETE", ["labour", "worker", "portfolio", id]) => {
            let user = require_user(user)?;
            let id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid portfolio id"))?;
            sqlx::query("DELETE FROM labour_worker_portfolio WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(user.id)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"deleted": true})));
        }
        ("GET", ["labour", "workers", "search"]) => {
            let category = query.get("category").cloned();
            let location_state = query.get("location_state").cloned();
            let location_city = query.get("location_city").cloned();
            let search = query.get("search").cloned();
            let limit = query
                .get("limit")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(20)
                .clamp(1, 50);
            let page = query
                .get("page")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(1)
                .max(1);
            let offset = (page - 1) * limit;
            let rows = sqlx::query("SELECT wp.*, u.full_name, u.email, u.username, u.document_verified, u.trust_score, p.avatar_url FROM labour_worker_profiles wp JOIN users u ON u.id = wp.user_id LEFT JOIN profiles p ON p.user_id = u.id WHERE ($1::text IS NULL OR wp.category = $1) AND ($2::text IS NULL OR wp.location_state = $2) AND ($3::text IS NULL OR wp.location_city = $3) AND ($4::text IS NULL OR u.full_name ILIKE ('%' || $4 || '%') OR wp.description ILIKE ('%' || $4 || '%')) ORDER BY wp.updated_at DESC LIMIT $5 OFFSET $6")
                .bind(category)
                .bind(location_state)
                .bind(location_city)
                .bind(search)
                .bind(limit)
                .bind(offset)
                .fetch_all(&state.pool)
                .await?;
            let data = rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "user_id": row.get::<Uuid, _>("user_id"), "category": row.get::<String, _>("category"), "experience_years": row.get::<i32, _>("experience_years"), "description": row.get::<String, _>("description"), "hourly_rate": row.get::<i64, _>("hourly_rate"), "daily_rate": row.get::<i64, _>("daily_rate"), "location_state": row.get::<String, _>("location_state"), "location_city": row.get::<String, _>("location_city"), "is_available": row.get::<bool, _>("is_available"), "skills": row.try_get::<Value, _>("skills").unwrap_or_else(|_| json!([])), "user": {"id": row.get::<Uuid, _>("user_id"), "name": row.get::<String, _>("full_name"), "email": row.get::<String, _>("email"), "username": row.try_get::<Option<String>, _>("username").ok().flatten(), "avatar_url": row.try_get::<Option<String>, _>("avatar_url").ok().flatten(), "trust_score": row.get::<i32, _>("trust_score"), "verified": row.get::<bool, _>("document_verified")}})).collect::<Vec<_>>();
            return Ok(ok(json!(data)));
        }
        ("GET", ["labour", "workers", id]) | ("GET", ["labour", "workers", id, "smart"]) => {
            let id = Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid worker id"))?;
            return Ok(ok(worker_profile_value(&state.pool, id).await?));
        }
        ("GET", ["labour", "jobs"]) => {
            let category = query.get("category").cloned();
            let limit = query
                .get("limit")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(20)
                .clamp(1, 50);
            let page = query
                .get("page")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(1)
                .max(1);
            let offset = (page - 1) * limit;
            let rows = sqlx::query("SELECT j.*, u.id AS employer_id, u.full_name, u.username, p.avatar_url FROM labour_jobs j JOIN users u ON u.id = j.employer_user_id LEFT JOIN profiles p ON p.user_id = u.id WHERE ($1::text IS NULL OR j.category = $1) ORDER BY j.created_at DESC LIMIT $2 OFFSET $3")
                .bind(category)
                .bind(limit)
                .bind(offset)
                .fetch_all(&state.pool)
                .await?;
            let jobs = rows.into_iter().map(|row| {
                let mut value = job_value(&row);
                if let Value::Object(ref mut map) = value {
                    map.insert("employer".into(), json!({"id": row.get::<Uuid, _>("employer_id"), "name": row.get::<String, _>("full_name"), "username": row.try_get::<Option<String>, _>("username").ok().flatten(), "avatar_url": row.try_get::<Option<String>, _>("avatar_url").ok().flatten()}));
                }
                value
            }).collect::<Vec<_>>();
            return Ok(ok(json!(jobs)));
        }
        ("POST", ["labour", "jobs"]) => {
            let user = require_user(user)?;
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO labour_jobs (id, employer_user_id, category, title, description, location_state, location_city, location_address, budget, estimated_duration_days, partial_payment_allowed, partial_payment_percentage, deadline) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)")
                .bind(id)
                .bind(user.id)
                .bind(str_field(&body, "category")?)
                .bind(str_field(&body, "title")?)
                .bind(str_field(&body, "description")?)
                .bind(str_field(&body, "location_state")?)
                .bind(str_field(&body, "location_city")?)
                .bind(str_field(&body, "location_address")?)
                .bind(i64_field(&body, "budget")?)
                .bind(i32_field(&body, "estimated_duration_days")?)
                .bind(bool_field(&body, "partial_payment_allowed", false))
                .bind(body.get("partial_payment_percentage").and_then(Value::as_f64))
                .bind(opt_str_field(&body, "deadline").and_then(|v| DateTime::parse_from_rfc3339(&v).ok()).map(|v| v.with_timezone(&Utc)))
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"id": id})));
        }
        ("GET", ["labour", "jobs", id]) => {
            let id = Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid job id"))?;
            let row = sqlx::query("SELECT j.*, u.id AS employer_id, u.full_name, u.username, p.avatar_url FROM labour_jobs j JOIN users u ON u.id = j.employer_user_id LEFT JOIN profiles p ON p.user_id = u.id WHERE j.id = $1")
                .bind(id)
                .fetch_optional(&state.pool)
                .await?;
            let Some(row) = row else {
                return Err(AppError::not_found("job not found"));
            };
            let applications = sqlx::query("SELECT a.id, a.job_id, a.worker_user_id, a.proposed_rate, a.estimated_completion, a.cover_letter, a.status, u.full_name, u.email, u.username FROM labour_job_applications a JOIN users u ON u.id = a.worker_user_id WHERE a.job_id = $1 ORDER BY a.created_at DESC")
                .bind(id)
                .fetch_all(&state.pool)
                .await?;
            let mut job = job_value(&row);
            if let Value::Object(ref mut map) = job {
                map.insert("employer".into(), json!({"id": row.get::<Uuid, _>("employer_id"), "name": row.get::<String, _>("full_name"), "username": row.try_get::<Option<String>, _>("username").ok().flatten(), "avatar_url": row.try_get::<Option<String>, _>("avatar_url").ok().flatten()}));
                map.insert("applications".into(), json!(applications.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "job_id": row.get::<Uuid, _>("job_id"), "worker_id": row.get::<Uuid, _>("worker_user_id"), "worker_user_id": row.get::<Uuid, _>("worker_user_id"), "proposed_rate": row.get::<i64, _>("proposed_rate"), "estimated_completion": row.get::<i32, _>("estimated_completion"), "cover_letter": row.get::<String, _>("cover_letter"), "status": row.get::<String, _>("status"), "worker": {"id": row.get::<Uuid, _>("worker_user_id"), "name": row.get::<String, _>("full_name"), "email": row.get::<String, _>("email"), "username": row.try_get::<Option<String>, _>("username").ok().flatten()}})).collect::<Vec<_>>()));
            }
            return Ok(ok(job));
        }
        ("POST", ["labour", "jobs", id, "applications"]) => {
            let user = require_user(user)?;
            let job_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid job id"))?;
            let application_id = Uuid::new_v4();
            sqlx::query("INSERT INTO labour_job_applications (id, job_id, worker_user_id, proposed_rate, estimated_completion, cover_letter) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (job_id, worker_user_id) DO UPDATE SET proposed_rate = EXCLUDED.proposed_rate, estimated_completion = EXCLUDED.estimated_completion, cover_letter = EXCLUDED.cover_letter, updated_at = NOW()")
                .bind(application_id)
                .bind(job_id)
                .bind(user.id)
                .bind(i64_field(&body, "proposed_rate")?)
                .bind(i32_field(&body, "estimated_completion")?)
                .bind(str_field(&body, "cover_letter")?)
                .execute(&state.pool)
                .await?;
            let employer_id = sqlx::query_scalar::<_, Uuid>(
                "SELECT employer_user_id FROM labour_jobs WHERE id = $1",
            )
            .bind(job_id)
            .fetch_one(&state.pool)
            .await?;
            create_notification(
                &state.pool,
                employer_id,
                "job_application",
                "New job application",
                "A worker applied to your job.",
                json!({"job_id": job_id}),
            )
            .await?;
            return Ok(ok(json!({"id": application_id})));
        }
        ("PUT", ["labour", "jobs", id, "assign"]) => {
            let user = require_user(user)?;
            let job_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid job id"))?;
            let worker_id = uuid_field(&body, "worker_id")?;
            let application = sqlx::query("SELECT id, proposed_rate, estimated_completion FROM labour_job_applications WHERE job_id = $1 AND worker_user_id = $2 ORDER BY created_at DESC LIMIT 1")
                .bind(job_id)
                .bind(worker_id)
                .fetch_optional(&state.pool)
                .await?;
            let Some(application) = application else {
                return Err(AppError::not_found("application not found"));
            };
            sqlx::query("UPDATE labour_job_applications SET status = 'accepted', updated_at = NOW() WHERE id = $1").bind(application.get::<Uuid, _>("id")).execute(&state.pool).await?;
            let contract_id = Uuid::new_v4();
            sqlx::query("INSERT INTO labour_contracts (id, job_id, employer_user_id, worker_user_id, application_id, agreed_rate, agreed_timeline, terms, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'active')")
                .bind(contract_id)
                .bind(job_id)
                .bind(user.id)
                .bind(worker_id)
                .bind(application.get::<Uuid, _>("id"))
                .bind(application.get::<i64, _>("proposed_rate"))
                .bind(application.get::<i32, _>("estimated_completion"))
                .bind("Assigned through application")
                .execute(&state.pool)
                .await?;
            sqlx::query(
                "UPDATE labour_jobs SET status = 'in_progress', updated_at = NOW() WHERE id = $1",
            )
            .bind(job_id)
            .execute(&state.pool)
            .await?;
            create_notification(
                &state.pool,
                worker_id,
                "contract_created",
                "You were hired",
                "An employer assigned you to a job.",
                json!({"job_id": job_id, "contract_id": contract_id}),
            )
            .await?;
            return Ok(ok(json!({"contract_id": contract_id})));
        }
        ("GET", ["labour", "jobs", id, "contract"]) => {
            let job_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid job id"))?;
            let row = sqlx::query("SELECT id, job_id, employer_user_id, worker_user_id, agreed_rate, agreed_timeline, terms, status, created_at FROM labour_contracts WHERE job_id = $1 ORDER BY created_at DESC LIMIT 1")
                .bind(job_id)
                .fetch_optional(&state.pool)
                .await?;
            return Ok(ok(row.map(|row| json!({"id": row.get::<Uuid, _>("id"), "job_id": row.get::<Uuid, _>("job_id"), "employer_user_id": row.get::<Uuid, _>("employer_user_id"), "worker_user_id": row.get::<Uuid, _>("worker_user_id"), "agreed_rate": row.get::<i64, _>("agreed_rate"), "agreed_timeline": row.get::<i32, _>("agreed_timeline"), "terms": row.get::<String, _>("terms"), "status": row.get::<String, _>("status"), "created_at": row.get::<DateTime<Utc>, _>("created_at")})).unwrap_or_else(|| json!(null))));
        }
        ("POST", ["labour", "jobs", id, "contract"]) => {
            let actor = require_user(user)?;
            let job_id =
                Some(Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid job id"))?);
            let worker_id = uuid_field(&body, "worker_id")?;
            let contract_id = Uuid::new_v4();
            sqlx::query("INSERT INTO labour_contracts (id, job_id, employer_user_id, worker_user_id, agreed_rate, agreed_timeline, terms, status) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active')")
                .bind(contract_id)
                .bind(job_id)
                .bind(actor.id)
                .bind(worker_id)
                .bind(i64_field(&body, "agreed_rate")?)
                .bind(i32_field(&body, "agreed_timeline")?)
                .bind(str_field(&body, "terms")?)
                .execute(&state.pool)
                .await?;
            if let Some(job_id) = job_id {
                sqlx::query("UPDATE labour_jobs SET status = 'in_progress', updated_at = NOW() WHERE id = $1").bind(job_id).execute(&state.pool).await?;
            }
            create_notification(
                &state.pool,
                worker_id,
                "contract_created",
                "Contract created",
                "A new contract was created for you.",
                json!({"contract_id": contract_id}),
            )
            .await?;
            return Ok(ok(json!({"id": contract_id})));
        }
        ("POST", ["labour", "jobs", "contract"]) => {
            let actor = require_user(user)?;
            let job_id = opt_str_field(&body, "job_id")
                .map(|v| Uuid::parse_str(&v).map_err(|_| AppError::bad_request("invalid job_id")))
                .transpose()?;
            let worker_id = uuid_field(&body, "worker_id")?;
            let contract_id = Uuid::new_v4();
            sqlx::query("INSERT INTO labour_contracts (id, job_id, employer_user_id, worker_user_id, agreed_rate, agreed_timeline, terms, status) VALUES (, , , , , , , active)")
                .bind(contract_id)
                .bind(job_id)
                .bind(actor.id)
                .bind(worker_id)
                .bind(i64_field(&body, "agreed_rate")?)
                .bind(i32_field(&body, "agreed_timeline")?)
                .bind(str_field(&body, "terms")?)
                .execute(&state.pool)
                .await?;
            if let Some(job_id) = job_id {
                sqlx::query(
                    "UPDATE labour_jobs SET status = in_progress, updated_at = NOW() WHERE id = ",
                )
                .bind(job_id)
                .execute(&state.pool)
                .await?;
            }
            create_notification(
                &state.pool,
                worker_id,
                "contract_created",
                "Contract created",
                "A new contract was created for you.",
                json!({"contract_id": contract_id}),
            )
            .await?;
            return Ok(ok(json!({"id": contract_id})));
        }
        ("GET", ["labour", "jobs", id, "progress"]) => {
            let job_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid job id"))?;
            let rows = sqlx::query("SELECT id, update_text, progress_percentage, created_at, user_id FROM labour_job_progress WHERE job_id = $1 ORDER BY created_at DESC")
                .bind(job_id)
                .fetch_all(&state.pool)
                .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "update_text": row.get::<String, _>("update_text"), "progress_percentage": row.get::<i32, _>("progress_percentage"), "created_at": row.get::<DateTime<Utc>, _>("created_at"), "user_id": row.get::<Uuid, _>("user_id")})).collect::<Vec<_>>())));
        }
        ("POST", ["labour", "jobs", id, "progress"]) => {
            let user = require_user(user)?;
            let job_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid job id"))?;
            sqlx::query("INSERT INTO labour_job_progress (id, job_id, contract_id, user_id, update_text, progress_percentage) VALUES ($1, $2, (SELECT id FROM labour_contracts WHERE job_id = $2 ORDER BY created_at DESC LIMIT 1), $3, $4, $5)")
                .bind(Uuid::new_v4())
                .bind(job_id)
                .bind(user.id)
                .bind(str_field(&body, "update_text")?)
                .bind(body.get("progress_percentage").and_then(Value::as_i64).unwrap_or(0) as i32)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"created": true})));
        }
        ("POST", ["labour", "jobs", id, "complete"]) => {
            let job_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid job id"))?;
            sqlx::query(
                "UPDATE labour_jobs SET status = 'completed', updated_at = NOW() WHERE id = $1",
            )
            .bind(job_id)
            .execute(&state.pool)
            .await?;
            sqlx::query("UPDATE labour_contracts SET status = 'completed', updated_at = NOW() WHERE job_id = $1").bind(job_id).execute(&state.pool).await?;
            return Ok(ok(json!({"completed": true})));
        }
        ("GET", ["labour", "contracts"]) => {
            let user = require_user(user)?;
            let status = query.get("status").cloned();
            let rows = sqlx::query("SELECT id, job_id, employer_user_id, worker_user_id, agreed_rate, agreed_timeline, terms, status, created_at FROM labour_contracts WHERE (employer_user_id = $1 OR worker_user_id = $1) AND ($2::text IS NULL OR status = $2) ORDER BY created_at DESC")
                .bind(user.id)
                .bind(status)
                .fetch_all(&state.pool)
                .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "job_id": row.try_get::<Option<Uuid>, _>("job_id").ok().flatten(), "employer_user_id": row.get::<Uuid, _>("employer_user_id"), "worker_user_id": row.get::<Uuid, _>("worker_user_id"), "agreed_rate": row.get::<i64, _>("agreed_rate"), "agreed_timeline": row.get::<i32, _>("agreed_timeline"), "terms": row.get::<String, _>("terms"), "status": row.get::<String, _>("status"), "created_at": row.get::<DateTime<Utc>, _>("created_at")})).collect::<Vec<_>>())));
        }
        ("GET", ["labour", "feed"]) => {
            let rows = sqlx::query("SELECT j.*, u.id AS employer_id, u.full_name, u.username, p.avatar_url FROM labour_jobs j JOIN users u ON u.id = j.employer_user_id LEFT JOIN profiles p ON p.user_id = u.id ORDER BY j.created_at DESC LIMIT 20")
                .fetch_all(&state.pool)
                .await?;
            let data = rows.into_iter().map(|row| {
                let mut value = job_value(&row);
                if let Value::Object(ref mut map) = value {
                    map.insert("employer".into(), json!({"id": row.get::<Uuid, _>("employer_id"), "name": row.get::<String, _>("full_name"), "username": row.try_get::<Option<String>, _>("username").ok().flatten(), "avatar_url": row.try_get::<Option<String>, _>("avatar_url").ok().flatten()}));
                }
                value
            }).collect::<Vec<_>>();
            return Ok(ok(json!(data)));
        }
        ("GET", ["labour", "worker", "reviews"]) => {
            let user = require_user(user)?;
            let reviews = sqlx::query("SELECT r.id, r.rating, r.comment, r.created_at, u.full_name, u.username FROM reviews r JOIN users u ON u.id = r.reviewer_id WHERE r.reviewee_id = $1 ORDER BY r.created_at DESC")
                .bind(user.id)
                .fetch_all(&state.pool)
                .await?;
            return Ok(ok(json!(reviews.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "rating": row.get::<i32, _>("rating"), "comment": row.get::<String, _>("comment"), "created_at": row.get::<DateTime<Utc>, _>("created_at"), "reviewer": {"name": row.get::<String, _>("full_name"), "username": row.try_get::<Option<String>, _>("username").ok().flatten()}})).collect::<Vec<_>>())));
        }
        ("GET", ["labour", "disputes", "my-disputes"]) => {
            let user = require_user(user)?;
            let rows = sqlx::query("SELECT * FROM disputes WHERE reporter_user_id = $1 OR subject_user_id = $1 ORDER BY created_at DESC")
                .bind(user.id)
                .fetch_all(&state.pool)
                .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "reference": row.get::<String, _>("reference"), "type": row.get::<String, _>("type"), "priority": row.get::<String, _>("priority"), "status": row.get::<String, _>("status"), "title": row.get::<String, _>("title"), "description": row.get::<String, _>("description"), "created_at": row.get::<DateTime<Utc>, _>("created_at")})).collect::<Vec<_>>())));
        }
        ("POST", ["labour", "disputes"]) => {
            let user = require_user(user)?;
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO disputes (id, reference, reporter_user_id, subject_user_id, type, priority, status, title, description) VALUES ($1, $2, $3, $4, $5, $6, 'open', $7, $8)")
                .bind(id)
                .bind(format!("DSP-{}", &id.to_string()[..8]))
                .bind(user.id)
                .bind(opt_str_field(&body, "subject_user_id").map(|v| Uuid::parse_str(&v).ok()).flatten())
                .bind(str_field(&body, "type")?)
                .bind(opt_str_field(&body, "priority").unwrap_or_else(|| "medium".to_string()))
                .bind(str_field(&body, "title")?)
                .bind(str_field(&body, "description")?)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"id": id})));
        }
        ("POST", ["labour", "jobs", id, "dispute"]) => {
            let user = require_user(user)?;
            let job_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid job id"))?;
            let dispute_id = Uuid::new_v4();
            sqlx::query("INSERT INTO disputes (id, reference, reporter_user_id, type, priority, status, title, description) VALUES ($1, $2, $3, 'quality', 'medium', 'open', $4, $5)")
                .bind(dispute_id)
                .bind(format!("DSP-{}", &dispute_id.to_string()[..8]))
                .bind(user.id)
                .bind(format!("Job dispute {job_id}"))
                .bind(str_field(&body, "description").unwrap_or_else(|_| "Job dispute opened".to_string()))
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"id": dispute_id})));
        }
        ("POST", ["labour", "disputes", id, "resolve"]) => {
            let _user = require_user(user)?;
            let id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid dispute id"))?;
            sqlx::query("UPDATE disputes SET status = 'resolved', resolved_at = NOW(), updated_at = NOW() WHERE id = $1").bind(id).execute(&state.pool).await?;
            return Ok(ok(json!({"resolved": true})));
        }
        ("GET", ["labour", "escrows"]) => {
            let user = require_user(user)?;
            let rows = sqlx::query("SELECT id, reference, amount, status, type, created_at FROM transactions WHERE user_id = $1 AND type IN ('escrow_hold', 'escrow_release') ORDER BY created_at DESC")
                .bind(user.id)
                .fetch_all(&state.pool)
                .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "reference": row.get::<String, _>("reference"), "amount": row.get::<i64, _>("amount"), "status": row.get::<String, _>("status"), "type": row.get::<String, _>("type"), "created_at": row.get::<DateTime<Utc>, _>("created_at")})).collect::<Vec<_>>())));
        }
        ("POST", ["labour", "jobs", id, "escrow", "release"]) => {
            let user = require_user(user)?;
            let job_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid job id"))?;
            let tx_id = Uuid::new_v4();
            sqlx::query("INSERT INTO transactions (id, reference, user_id, type, amount, currency, status, metadata_json, created_at, updated_at) VALUES ($1, $2, $3, 'escrow_release', $4, 'NGN', 'succeeded', $5, NOW(), NOW())")
                .bind(tx_id)
                .bind(format!("ESC-{}", &tx_id.to_string()[..8]))
                .bind(user.id)
                .bind(0_i64)
                .bind(json!({"job_id": job_id}))
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"released": true})));
        }
        ("GET", ["chat", "chats"]) => {
            let user = require_user(user)?;
            let rows = sqlx::query("SELECT c.id, c.participant_one_id, c.participant_two_id, c.job_id, c.last_message_at, u.id AS other_user_id, u.full_name, u.username, p.avatar_url, (SELECT COUNT(*) FROM chat_messages m WHERE m.chat_id = c.id AND m.sender_id <> $1 AND m.is_read = FALSE) AS unread_count, (SELECT content FROM chat_messages m WHERE m.chat_id = c.id ORDER BY created_at DESC LIMIT 1) AS last_message, (SELECT created_at FROM chat_messages m WHERE m.chat_id = c.id ORDER BY created_at DESC LIMIT 1) AS last_message_created FROM chat_chats c JOIN users u ON u.id = CASE WHEN c.participant_one_id = $1 THEN c.participant_two_id ELSE c.participant_one_id END LEFT JOIN profiles p ON p.user_id = u.id WHERE c.participant_one_id = $1 OR c.participant_two_id = $1 ORDER BY c.last_message_at DESC")
                .bind(user.id)
                .fetch_all(&state.pool)
                .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "participant_one_id": row.get::<Uuid, _>("participant_one_id"), "participant_two_id": row.get::<Uuid, _>("participant_two_id"), "job_id": row.try_get::<Option<Uuid>, _>("job_id").ok().flatten(), "last_message_at": row.get::<DateTime<Utc>, _>("last_message_at"), "other_user": {"id": row.get::<Uuid, _>("other_user_id"), "name": row.get::<String, _>("full_name"), "username": row.try_get::<Option<String>, _>("username").ok().flatten(), "avatar_url": row.try_get::<Option<String>, _>("avatar_url").ok().flatten()}, "last_message": row.try_get::<Option<String>, _>("last_message").ok().flatten().map(|content| json!({"content": content, "created_at": row.try_get::<Option<DateTime<Utc>>, _>("last_message_created").ok().flatten(), "is_read": true})), "unread_count": row.get::<i64, _>("unread_count")})).collect::<Vec<_>>())));
        }
        ("POST", ["chat", "chats"]) => {
            let user = require_user(user)?;
            let other_user_id = uuid_field(&body, "other_user_id")?;
            let job_id = opt_str_field(&body, "job_id")
                .map(|v| Uuid::parse_str(&v).ok())
                .flatten();
            let pair = if user.id < other_user_id {
                (user.id, other_user_id)
            } else {
                (other_user_id, user.id)
            };
            if let Some(existing) = sqlx::query_scalar::<_, Option<Uuid>>("SELECT id FROM chat_chats WHERE participant_one_id = $1 AND participant_two_id = $2 AND (job_id = $3 OR (job_id IS NULL AND $3 IS NULL)) LIMIT 1")
                .bind(pair.0)
                .bind(pair.1)
                .bind(job_id)
                .fetch_one(&state.pool)
                .await? {
                return Ok(ok(json!({"chat": {"id": existing}})));
            }
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO chat_chats (id, participant_one_id, participant_two_id, job_id, last_message_at, created_at) VALUES ($1, $2, $3, $4, NOW(), NOW())")
                .bind(id)
                .bind(pair.0)
                .bind(pair.1)
                .bind(job_id)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"chat": {"id": id}})));
        }
        ("GET", ["chat", "chats", id, "messages"]) => {
            let user = require_user(user)?;
            let chat_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid chat id"))?;
            let owns = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM chat_chats WHERE id = $1 AND (participant_one_id = $2 OR participant_two_id = $2)")
                .bind(chat_id)
                .bind(user.id)
                .fetch_one(&state.pool)
                .await?;
            if owns == 0 {
                return Err(AppError::forbidden("chat not found"));
            }
            let rows = sqlx::query("SELECT id, chat_id, sender_id, content, message_type, created_at, is_read FROM chat_messages WHERE chat_id = $1 ORDER BY created_at ASC")
                .bind(chat_id)
                .fetch_all(&state.pool)
                .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "chat_id": row.get::<Uuid, _>("chat_id"), "sender_id": row.get::<Uuid, _>("sender_id"), "content": row.get::<String, _>("content"), "message_type": row.get::<String, _>("message_type"), "created_at": row.get::<DateTime<Utc>, _>("created_at"), "is_read": row.get::<bool, _>("is_read")})).collect::<Vec<_>>())));
        }
        ("POST", ["chat", "chats", id, "messages"]) => {
            let user = require_user(user)?;
            let chat_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid chat id"))?;
            let message_id = Uuid::new_v4();
            sqlx::query("INSERT INTO chat_messages (id, chat_id, sender_id, content, message_type, is_read, created_at) VALUES ($1, $2, $3, $4, $5, FALSE, NOW())")
                .bind(message_id)
                .bind(chat_id)
                .bind(user.id)
                .bind(str_field(&body, "content")?)
                .bind(opt_str_field(&body, "message_type").unwrap_or_else(|| "text".to_string()))
                .execute(&state.pool)
                .await?;
            sqlx::query("UPDATE chat_chats SET last_message_at = NOW() WHERE id = $1")
                .bind(chat_id)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"id": message_id})));
        }
        ("POST", ["chat", "chats", id, "read"]) => {
            let user = require_user(user)?;
            let chat_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid chat id"))?;
            sqlx::query(
                "UPDATE chat_messages SET is_read = TRUE WHERE chat_id = $1 AND sender_id <> $2",
            )
            .bind(chat_id)
            .bind(user.id)
            .execute(&state.pool)
            .await?;
            return Ok(ok(json!({"read": true})));
        }
        ("GET", ["chat", "unread-count"]) => {
            let user = require_user(user)?;
            let unread = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM chat_messages m JOIN chat_chats c ON c.id = m.chat_id WHERE (c.participant_one_id = $1 OR c.participant_two_id = $1) AND m.sender_id <> $1 AND m.is_read = FALSE")
                .bind(user.id)
                .fetch_one(&state.pool)
                .await?;
            return Ok(ok(json!({"unread_count": unread})));
        }
        ("GET", ["support", "my-tickets"]) => {
            let user = require_user(user)?;
            let rows = sqlx::query("SELECT id, title, description, category, priority, status, created_at FROM support_tickets WHERE user_id = $1 ORDER BY created_at DESC")
                .bind(user.id)
                .fetch_all(&state.pool)
                .await?;
            let data = rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "title": row.get::<String, _>("title"), "description": row.get::<String, _>("description"), "category": row.get::<String, _>("category"), "priority": row.get::<String, _>("priority"), "status": row.get::<String, _>("status"), "created_at": row.get::<DateTime<Utc>, _>("created_at"), "messages": []})).collect::<Vec<_>>();
            return Ok(ok(json!(data)));
        }
        ("GET", ["support", "tickets"]) => {
            let actor = require_user(user)?;
            if actor.role != UserRole::Admin {
                return Ok(ok(json!([])));
            }
            let rows = sqlx::query("SELECT id, title, description, category, priority, status, created_at FROM support_tickets ORDER BY created_at DESC")
                .fetch_all(&state.pool)
                .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "title": row.get::<String, _>("title"), "description": row.get::<String, _>("description"), "category": row.get::<String, _>("category"), "priority": row.get::<String, _>("priority"), "status": row.get::<String, _>("status"), "created_at": row.get::<DateTime<Utc>, _>("created_at")})).collect::<Vec<_>>())));
        }
        ("POST", ["support", "tickets"]) => {
            let user = require_user(user)?;
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO support_tickets (id, user_id, title, description, category, priority, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, 'open', NOW(), NOW())")
                .bind(id)
                .bind(user.id)
                .bind(str_field(&body, "title")?)
                .bind(str_field(&body, "description")?)
                .bind(str_field(&body, "category")?)
                .bind(str_field(&body, "priority")?)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"id": id})));
        }
        ("POST", ["support", "tickets", id, "assign"]) => {
            let actor = require_user(user)?;
            require_admin(&actor)?;
            let ticket_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid ticket id"))?;
            let assigned_to = opt_str_field(&body, "assigned_to")
                .map(|v| Uuid::parse_str(&v).ok())
                .flatten();
            sqlx::query(
                "UPDATE support_tickets SET assigned_to = $2, updated_at = NOW() WHERE id = $1",
            )
            .bind(ticket_id)
            .bind(assigned_to)
            .execute(&state.pool)
            .await?;
            return Ok(ok(json!({"assigned": true})));
        }
        ("POST", ["support", "tickets", id, "status"]) => {
            let actor = require_user(user)?;
            if actor.role != UserRole::Admin {
                return Err(AppError::forbidden("admin access required"));
            }
            let ticket_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid ticket id"))?;
            sqlx::query("UPDATE support_tickets SET status = $2, updated_at = NOW() WHERE id = $1")
                .bind(ticket_id)
                .bind(str_field(&body, "status")?)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"updated": true})));
        }
        ("POST", ["support", "tickets", id, "messages"]) => {
            let actor = require_user(user)?;
            let ticket_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid ticket id"))?;
            sqlx::query("INSERT INTO support_ticket_messages (id, ticket_id, user_id, message, is_internal, created_at) VALUES ($1, $2, $3, $4, $5, NOW())")
                .bind(Uuid::new_v4())
                .bind(ticket_id)
                .bind(actor.id)
                .bind(str_field(&body, "message")?)
                .bind(bool_field(&body, "is_internal", false))
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"sent": true})));
        }
        ("GET", ["support", "stats"]) => {
            let actor = require_user(user)?;
            if actor.role != UserRole::Admin {
                return Ok(ok(
                    json!({"open": 0, "in_progress": 0, "resolved": 0, "closed": 0}),
                ));
            }
            let open = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM support_tickets WHERE status = 'open'",
            )
            .fetch_one(&state.pool)
            .await?;
            let in_progress = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM support_tickets WHERE status = 'in_progress'",
            )
            .fetch_one(&state.pool)
            .await?;
            let resolved = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM support_tickets WHERE status = 'resolved'",
            )
            .fetch_one(&state.pool)
            .await?;
            let closed = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM support_tickets WHERE status = 'closed'",
            )
            .fetch_one(&state.pool)
            .await?;
            return Ok(ok(
                json!({"open": open, "in_progress": in_progress, "resolved": resolved, "closed": closed}),
            ));
        }
        ("GET", ["notifications"]) => {
            let user = require_user(user)?;
            let page = query
                .get("page")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(1);
            let limit = query
                .get("limit")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(20);
            return Ok(top(list_notifications_response(
                &state.pool,
                user.id,
                page,
                limit,
            )
            .await?));
        }
        ("GET", ["notifications", "unread-count"]) => {
            let user = require_user(user)?;
            let unread = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND read_at IS NULL",
            )
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
            return Ok(top(
                json!({"unread_count": unread, "data": {"unread_count": unread}}),
            ));
        }
        ("POST", ["notifications", "read"]) => {
            let user = require_user(user)?;
            let ids = body
                .get("notification_ids")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for id in ids {
                if let Some(id) = id.as_str().and_then(|v| Uuid::parse_str(v).ok()) {
                    sqlx::query(
                        "UPDATE notifications SET read_at = NOW() WHERE id = $1 AND user_id = $2",
                    )
                    .bind(id)
                    .bind(user.id)
                    .execute(&state.pool)
                    .await?;
                }
            }
            return Ok(ok(json!({"updated": true})));
        }
        ("POST", ["notifications", "read-all"]) => {
            let user = require_user(user)?;
            sqlx::query(
                "UPDATE notifications SET read_at = NOW() WHERE user_id = $1 AND read_at IS NULL",
            )
            .bind(user.id)
            .execute(&state.pool)
            .await?;
            return Ok(ok(json!({"updated": true})));
        }
        ("PUT", ["notifications", id, "read"]) => {
            let user = require_user(user)?;
            let id = Uuid::parse_str(id)
                .map_err(|_| AppError::bad_request("invalid notification id"))?;
            sqlx::query("UPDATE notifications SET read_at = NOW() WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(user.id)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"updated": true})));
        }
        ("DELETE", ["notifications", id]) => {
            let user = require_user(user)?;
            let id = Uuid::parse_str(id)
                .map_err(|_| AppError::bad_request("invalid notification id"))?;
            sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(user.id)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"deleted": true})));
        }
        ("GET", ["wallet"]) => {
            let user = require_user(user)?;
            return Ok(ok(wallet_summary(&state.pool, user.id).await?));
        }
        ("POST", ["wallet", "create"]) => {
            let user = require_user(user)?;
            let wallet_address = format!("wallet_{}", &user.id.to_string()[..12]);
            sqlx::query("UPDATE users SET wallet_address = COALESCE(wallet_address, $2), updated_at = NOW() WHERE id = $1")
                .bind(user.id)
                .bind(wallet_address)
                .execute(&state.pool)
                .await?;
            return Ok(ok(wallet_summary(&state.pool, user.id).await?));
        }
        ("GET", ["wallet", "bank-accounts"]) => {
            let user = require_user(user)?;
            let rows = sqlx::query("SELECT id, account_name, account_number, bank_code, is_primary, created_at FROM wallet_bank_accounts WHERE user_id = $1 ORDER BY is_primary DESC, created_at DESC")
                .bind(user.id)
                .fetch_all(&state.pool)
                .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "account_name": row.get::<String, _>("account_name"), "account_number": row.get::<String, _>("account_number"), "bank_name": row.get::<String, _>("bank_code"), "bank_code": row.get::<String, _>("bank_code"), "is_primary": row.get::<bool, _>("is_primary"), "verified": true})).collect::<Vec<_>>())));
        }
        ("POST", ["wallet", "bank-accounts"]) => {
            let user = require_user(user)?;
            let id = Uuid::new_v4();
            let has_any = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM wallet_bank_accounts WHERE user_id = $1",
            )
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?
                > 0;
            sqlx::query("INSERT INTO wallet_bank_accounts (id, user_id, account_name, account_number, bank_code, is_primary, created_at) VALUES ($1, $2, $3, $4, $5, $6, NOW())")
                .bind(id)
                .bind(user.id)
                .bind(str_field(&body, "account_name")?)
                .bind(str_field(&body, "account_number")?)
                .bind(str_field(&body, "bank_code")?)
                .bind(!has_any)
                .execute(&state.pool)
                .await?;
            sqlx::query(
                "UPDATE users SET bank_account_linked = TRUE, updated_at = NOW() WHERE id = $1",
            )
            .bind(user.id)
            .execute(&state.pool)
            .await?;
            return Ok(ok(json!({"id": id})));
        }
        ("PUT", ["wallet", "bank-accounts", id, "primary"]) => {
            let user = require_user(user)?;
            let id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid account id"))?;
            sqlx::query("UPDATE wallet_bank_accounts SET is_primary = FALSE WHERE user_id = $1")
                .bind(user.id)
                .execute(&state.pool)
                .await?;
            sqlx::query(
                "UPDATE wallet_bank_accounts SET is_primary = TRUE WHERE id = $1 AND user_id = $2",
            )
            .bind(id)
            .bind(user.id)
            .execute(&state.pool)
            .await?;
            return Ok(ok(json!({"updated": true})));
        }
        ("GET", ["wallet", "transactions"]) => {
            let user = require_user(user)?;
            let rows = sqlx::query("SELECT id, reference, amount, status, type, provider, provider_reference, metadata_json, created_at FROM transactions WHERE user_id = $1 ORDER BY created_at DESC")
                .bind(user.id)
                .fetch_all(&state.pool)
                .await?;
            let balance = wallet_summary(&state.pool, user.id).await?;
            let current_balance = balance.get("balance").and_then(Value::as_i64).unwrap_or(0);
            return Ok(ok(
                json!({"transactions": rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "reference": row.get::<String, _>("reference"), "amount": row.get::<i64, _>("amount"), "type": row.get::<String, _>("type"), "status": row.get::<String, _>("status"), "description": row.try_get::<Option<String>, _>("provider").ok().flatten().unwrap_or_else(|| row.get::<String, _>("type")), "created_at": row.get::<DateTime<Utc>, _>("created_at"), "balance_before": current_balance, "balance_after": current_balance, "metadata": row.try_get::<Value, _>("metadata_json").unwrap_or_else(|_| json!({}))})).collect::<Vec<_>>() }),
            ));
        }
        ("POST", ["wallet", "deposit"]) => {
            let user = require_user(user)?;
            let id = Uuid::new_v4();
            let amount = i64_field(&body, "amount")?;
            let reference = format!("DEP-{}", &id.to_string()[..8]);
            sqlx::query("INSERT INTO transactions (id, reference, user_id, type, amount, currency, status, provider, metadata_json, created_at, updated_at) VALUES ($1, $2, $3, 'charge', $4, 'NGN', 'pending', $5, $6, NOW(), NOW())")
                .bind(id)
                .bind(&reference)
                .bind(user.id)
                .bind(amount)
                .bind(opt_str_field(&body, "payment_method").unwrap_or_else(|| "card".to_string()))
                .bind(body.get("metadata").cloned().unwrap_or_else(|| json!({})))
                .execute(&state.pool)
                .await?;
            return Ok(ok(
                json!({"reference": reference, "payment_url": format!("{PUBLIC_APP_BASE_URL}/payment/verify?reference={reference}")}),
            ));
        }
        ("POST", ["wallet", "deposit", "verify"]) => {
            let user = require_user(user)?;
            let reference = opt_str_field(&body, "reference")
                .or_else(|| opt_str_field(&body, "provider_reference"))
                .ok_or_else(|| AppError::bad_request("reference is required"))?;
            sqlx::query("UPDATE transactions SET status = 'succeeded', updated_at = NOW() WHERE reference = $1 AND user_id = $2")
                .bind(&reference)
                .bind(user.id)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"verified": true})));
        }
        ("POST", ["wallet", "withdraw"]) => {
            let user = require_user(user)?;
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO transactions (id, reference, user_id, type, amount, currency, status, provider, metadata_json, created_at, updated_at) VALUES ($1, $2, $3, 'payout', $4, 'NGN', 'processing', 'bank_transfer', $5, NOW(), NOW())")
                .bind(id)
                .bind(format!("WDL-{}", &id.to_string()[..8]))
                .bind(user.id)
                .bind(i64_field(&body, "amount")?)
                .bind(json!({"bank_account_id": opt_str_field(&body, "bank_account_id")}))
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"requested": true})));
        }
        ("POST", ["wallet", "transfer"]) => {
            let user = require_user(user)?;
            let id = Uuid::new_v4();
            let recipient = str_field(&body, "recipient_identifier")?;
            sqlx::query("INSERT INTO transactions (id, reference, user_id, type, amount, currency, status, provider, metadata_json, created_at, updated_at) VALUES ($1, $2, $3, 'fee', $4, 'NGN', 'succeeded', 'internal_transfer', $5, NOW(), NOW())")
                .bind(id)
                .bind(format!("TRF-{}", &id.to_string()[..8]))
                .bind(user.id)
                .bind(i64_field(&body, "amount")?)
                .bind(json!({"recipient_identifier": recipient, "description": opt_str_field(&body, "description")}))
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"transferred": true})));
        }
        ("GET", ["wallet", "naira"]) => {
            let user = require_user(user)?;
            let wallet = wallet_summary(&state.pool, user.id).await?;
            return Ok(ok(
                json!({"available_balance": wallet.get("balance").cloned().unwrap_or_else(|| json!(0))}),
            ));
        }
        ("POST", ["vendor", "profile"]) | ("PUT", ["vendor", "profile"]) => {
            let user = require_user(user)?;
            sqlx::query("INSERT INTO vendor_profiles (id, user_id, business_name, description, location_state, location_city, created_at, updated_at) VALUES ((SELECT COALESCE((SELECT id FROM vendor_profiles WHERE user_id = $1 LIMIT 1), $2)), $1, $3, $4, $5, $6, NOW(), NOW()) ON CONFLICT (user_id) DO UPDATE SET business_name = EXCLUDED.business_name, description = EXCLUDED.description, location_state = EXCLUDED.location_state, location_city = EXCLUDED.location_city, updated_at = NOW()")
                .bind(user.id)
                .bind(Uuid::new_v4())
                .bind(str_field(&body, "business_name")?)
                .bind(opt_str_field(&body, "description"))
                .bind(str_field(&body, "location_state")?)
                .bind(str_field(&body, "location_city")?)
                .execute(&state.pool)
                .await?;
            let row = sqlx::query("SELECT * FROM vendor_profiles WHERE user_id = $1")
                .bind(user.id)
                .fetch_one(&state.pool)
                .await?;
            return Ok(ok(
                json!({"id": row.get::<Uuid, _>("id"), "user_id": row.get::<Uuid, _>("user_id"), "business_name": row.get::<String, _>("business_name"), "description": row.try_get::<Option<String>, _>("description").ok().flatten(), "location_state": row.get::<String, _>("location_state"), "location_city": row.get::<String, _>("location_city")}),
            ));
        }
        ("GET", ["vendor", "profile"]) => {
            let user = require_user(user)?;
            let row = sqlx::query("SELECT * FROM vendor_profiles WHERE user_id = $1")
                .bind(user.id)
                .fetch_optional(&state.pool)
                .await?;
            return Ok(ok(row.map(|row| json!({"id": row.get::<Uuid, _>("id"), "user_id": row.get::<Uuid, _>("user_id"), "business_name": row.get::<String, _>("business_name"), "description": row.try_get::<Option<String>, _>("description").ok().flatten(), "location_state": row.get::<String, _>("location_state"), "location_city": row.get::<String, _>("location_city")})).unwrap_or_else(|| json!(null))));
        }
        ("POST", ["vendor", "services"]) => {
            let user = require_user(user)?;
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO vendor_services (id, vendor_user_id, title, description, category, price, location_state, location_city, image_urls, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active', NOW(), NOW())")
                .bind(id)
                .bind(user.id)
                .bind(str_field(&body, "title")?)
                .bind(str_field(&body, "description")?)
                .bind(str_field(&body, "category")?)
                .bind(i64_field(&body, "price")?)
                .bind(opt_str_field(&body, "location_state"))
                .bind(opt_str_field(&body, "location_city"))
                .bind(body.get("image_urls").cloned().unwrap_or_else(|| json!([])))
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"id": id})));
        }
        ("GET", ["vendor", "services"]) => {
            let user = require_user(user)?;
            let rows = sqlx::query(
                "SELECT * FROM vendor_services WHERE vendor_user_id = $1 ORDER BY created_at DESC",
            )
            .bind(user.id)
            .fetch_all(&state.pool)
            .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "vendor_user_id": row.get::<Uuid, _>("vendor_user_id"), "title": row.get::<String, _>("title"), "description": row.get::<String, _>("description"), "category": row.get::<String, _>("category"), "price": row.get::<i64, _>("price"), "location_state": row.try_get::<Option<String>, _>("location_state").ok().flatten(), "location_city": row.try_get::<Option<String>, _>("location_city").ok().flatten(), "image_urls": row.try_get::<Value, _>("image_urls").unwrap_or_else(|_| json!([])), "status": row.get::<String, _>("status"), "created_at": row.get::<DateTime<Utc>, _>("created_at")})).collect::<Vec<_>>())));
        }
        ("POST", ["services", id, "purchase"]) => {
            let user = require_user(user)?;
            let service_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid service id"))?;
            let vendor_id = sqlx::query_scalar::<_, Uuid>(
                "SELECT vendor_user_id FROM vendor_services WHERE id = $1",
            )
            .bind(service_id)
            .fetch_one(&state.pool)
            .await?;
            let amount = body
                .get("amount")
                .and_then(Value::as_i64)
                .unwrap_or_else(|| 0);
            let order_id = Uuid::new_v4();
            sqlx::query("INSERT INTO vendor_orders (id, service_id, buyer_user_id, vendor_user_id, amount, status, created_at) VALUES ($1, $2, $3, $4, $5, 'paid', NOW())")
                .bind(order_id)
                .bind(service_id)
                .bind(user.id)
                .bind(vendor_id)
                .bind(amount)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"id": order_id})));
        }
        ("POST", ["services", id, "inquiry"]) => {
            let user = require_user(user)?;
            let service_id =
                Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid service id"))?;
            let vendor_id = sqlx::query_scalar::<_, Uuid>(
                "SELECT vendor_user_id FROM vendor_services WHERE id = $1",
            )
            .bind(service_id)
            .fetch_one(&state.pool)
            .await?;
            let inquiry_id = Uuid::new_v4();
            sqlx::query("INSERT INTO vendor_inquiries (id, service_id, sender_user_id, vendor_user_id, message, status, created_at) VALUES ($1, $2, $3, $4, $5, 'open', NOW())")
                .bind(inquiry_id)
                .bind(service_id)
                .bind(user.id)
                .bind(vendor_id)
                .bind(str_field(&body, "message")?)
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"id": inquiry_id})));
        }
        ("GET", ["orders", "my-purchases"]) => {
            let user = require_user(user)?;
            let rows = sqlx::query(
                "SELECT * FROM vendor_orders WHERE buyer_user_id = $1 ORDER BY created_at DESC",
            )
            .bind(user.id)
            .fetch_all(&state.pool)
            .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "service_id": row.get::<Uuid, _>("service_id"), "buyer_user_id": row.get::<Uuid, _>("buyer_user_id"), "vendor_user_id": row.get::<Uuid, _>("vendor_user_id"), "amount": row.get::<i64, _>("amount"), "status": row.get::<String, _>("status"), "created_at": row.get::<DateTime<Utc>, _>("created_at"), "completed_at": row.try_get::<Option<DateTime<Utc>>, _>("completed_at").ok().flatten()})).collect::<Vec<_>>())));
        }
        ("GET", ["vendor", "orders"]) => {
            let user = require_user(user)?;
            let rows = sqlx::query(
                "SELECT * FROM vendor_orders WHERE vendor_user_id = $1 ORDER BY created_at DESC",
            )
            .bind(user.id)
            .fetch_all(&state.pool)
            .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "service_id": row.get::<Uuid, _>("service_id"), "buyer_user_id": row.get::<Uuid, _>("buyer_user_id"), "vendor_user_id": row.get::<Uuid, _>("vendor_user_id"), "amount": row.get::<i64, _>("amount"), "status": row.get::<String, _>("status"), "created_at": row.get::<DateTime<Utc>, _>("created_at"), "completed_at": row.try_get::<Option<DateTime<Utc>>, _>("completed_at").ok().flatten()})).collect::<Vec<_>>())));
        }
        ("GET", ["orders", id]) => {
            let id = Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid order id"))?;
            let row = sqlx::query("SELECT * FROM vendor_orders WHERE id = $1")
                .bind(id)
                .fetch_optional(&state.pool)
                .await?;
            return Ok(ok(row.map(|row| json!({"id": row.get::<Uuid, _>("id"), "service_id": row.get::<Uuid, _>("service_id"), "buyer_user_id": row.get::<Uuid, _>("buyer_user_id"), "vendor_user_id": row.get::<Uuid, _>("vendor_user_id"), "amount": row.get::<i64, _>("amount"), "status": row.get::<String, _>("status"), "rating": row.try_get::<Option<i32>, _>("rating").ok().flatten(), "review_comment": row.try_get::<Option<String>, _>("review_comment").ok().flatten(), "created_at": row.get::<DateTime<Utc>, _>("created_at")})).unwrap_or_else(|| json!(null))));
        }
        ("POST", ["orders", id, "complete"]) => {
            let _user = require_user(user)?;
            let id = Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid order id"))?;
            sqlx::query("UPDATE vendor_orders SET status = 'completed', rating = $2, review_comment = $3, completed_at = NOW() WHERE id = $1")
                .bind(id)
                .bind(body.get("rating").and_then(Value::as_i64).map(|v| v as i32))
                .bind(opt_str_field(&body, "review_comment"))
                .execute(&state.pool)
                .await?;
            return Ok(ok(json!({"completed": true})));
        }
        ("GET", ["vendor", "analytics"]) => {
            let user = require_user(user)?;
            let services = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM vendor_services WHERE vendor_user_id = $1",
            )
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
            let orders = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM vendor_orders WHERE vendor_user_id = $1",
            )
            .bind(user.id)
            .fetch_one(&state.pool)
            .await?;
            let revenue = sqlx::query_scalar::<_, Option<i64>>("SELECT COALESCE(SUM(amount),0) FROM vendor_orders WHERE vendor_user_id = $1 AND status IN ('paid','completed')").bind(user.id).fetch_one(&state.pool).await?.unwrap_or(0);
            return Ok(ok(
                json!({"services": services, "orders": orders, "revenue": revenue}),
            ));
        }
        ("GET", ["vendor", "inquiries"]) => {
            let user = require_user(user)?;
            let rows = sqlx::query(
                "SELECT * FROM vendor_inquiries WHERE vendor_user_id = $1 ORDER BY created_at DESC",
            )
            .bind(user.id)
            .fetch_all(&state.pool)
            .await?;
            return Ok(ok(json!(rows.into_iter().map(|row| json!({"id": row.get::<Uuid, _>("id"), "service_id": row.get::<Uuid, _>("service_id"), "sender_user_id": row.get::<Uuid, _>("sender_user_id"), "vendor_user_id": row.get::<Uuid, _>("vendor_user_id"), "message": row.get::<String, _>("message"), "status": row.get::<String, _>("status"), "created_at": row.get::<DateTime<Utc>, _>("created_at")})).collect::<Vec<_>>())));
        }
        ("GET", ["users", "admin", "users"]) => {
            let actor = require_user(user)?;
            require_admin(&actor)?;
            let rows = sqlx::query("SELECT id FROM users ORDER BY created_at DESC")
                .fetch_all(&state.pool)
                .await?;
            let mut users = Vec::with_capacity(rows.len());
            for row in rows {
                users.push(frontend_user_json(&state.pool, row.get::<Uuid, _>("id")).await?);
            }
            return Ok(ok(json!(users)));
        }
        ("GET", ["users", "admin", "users", id]) => {
            let actor = require_user(user)?;
            require_admin(&actor)?;
            let id = Uuid::parse_str(id).map_err(|_| AppError::bad_request("invalid user id"))?;
            return Ok(ok(frontend_user_json(&state.pool, id).await?));
        }
        _ => {
            let _ = uri;
            Err(AppError::not_found("endpoint not found"))
        }
    }
}
