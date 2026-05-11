use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    domain::trust::{CreateReportInput, CreateReviewInput, ModerateReportInput},
    infrastructure::email::service::{header_asset_url, HEADER_REVIEW},
    interfaces::http::{errors::AppError, middleware::auth::AuthUser, state::AppState},
};

pub async fn create_review(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateReviewInput>,
) -> Result<(StatusCode, Json<crate::domain::trust::Review>), AppError> {
    let review = state.trust_use_cases.create_review(&user, payload).await?;
    Ok((StatusCode::CREATED, Json(review)))
}

pub async fn list_user_reviews(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<crate::domain::trust::ReviewView>>, AppError> {
    let reviews = state.trust_use_cases.list_reviews_for_user(user_id).await?;
    Ok(Json(reviews))
}

pub async fn create_report(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateReportInput>,
) -> Result<(StatusCode, Json<crate::domain::trust::Report>), AppError> {
    let report = state.trust_use_cases.create_report(&user, payload).await?;
    Ok((StatusCode::CREATED, Json(report)))
}

pub async fn moderate_report(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(report_id): Path<Uuid>,
    Json(payload): Json<ModerateReportInput>,
) -> Result<Json<crate::domain::trust::Report>, AppError> {
    if !user.role.can_moderate() {
        return Err(AppError::forbidden("only admins can moderate reports"));
    }
    let property_action = payload.property_action.clone();
    let review_notes = payload.review_notes.clone();
    let report = state
        .trust_use_cases
        .moderate_report(&user, report_id, payload)
        .await?;

    if report.status == "upheld" {
        if let (Some(property_id), Some(action)) = (report.property_id, property_action.as_deref()) {
            let property_repo = crate::domain::properties::PropertyRepository::new(state.pool.clone());
            match action {
                "hide" => {
                    property_repo.hide_property(property_id).await?;
                }
                "suspend" => {
                    property_repo.suspend_property(property_id).await?;
                }
                _ => {}
            }

            let property_row = sqlx::query(
                r#"
                SELECT
                    p.id,
                    p.title,
                    p.location,
                    COALESCE(p.agent_id, p.owner_id) AS recipient_id,
                    CASE WHEN p.agent_id IS NOT NULL THEN 'agent' ELSE 'landlord' END AS recipient_role,
                    recipient.full_name AS recipient_name,
                    recipient.email AS recipient_email
                FROM properties p
                INNER JOIN users recipient ON recipient.id = COALESCE(p.agent_id, p.owner_id)
                WHERE p.id = $1
                "#,
            )
            .bind(property_id)
            .fetch_optional(&state.pool)
            .await?;

            if let Some(row) = property_row {
                let recipient_id = row.get::<Uuid, _>("recipient_id");
                let recipient_role = row.get::<String, _>("recipient_role");
                let recipient_name = row.get::<String, _>("recipient_name");
                let recipient_email = row.get::<String, _>("recipient_email");
                let property_title = row.get::<String, _>("title");
                let property_location = row.get::<String, _>("location");
                let action_label = match action {
                    "hide" => "hidden from public search",
                    "suspend" => "suspended",
                    _ => "reviewed",
                };
                let action_url = if recipient_role == "agent" {
                    format!("/provider/listings/{property_id}")
                } else {
                    "/landlord/properties".to_string()
                };

                sqlx::query(
                    "INSERT INTO notifications (id, user_id, type, title, body, data_json) VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(Uuid::new_v4())
                .bind(recipient_id)
                .bind("property_moderation")
                .bind("Listing moderation update")
                .bind(format!(
                    "Your listing \"{property_title}\" was {action_label}. Reason: {review_notes}"
                ))
                .bind(json!({
                    "actionUrl": action_url.clone(),
                    "propertyId": property_id,
                    "reportId": report.id,
                    "action": action,
                }))
                .execute(&state.pool)
                .await?;

                let email = state.mail_service.property_moderation_email(
                    recipient_email,
                    &recipient_name,
                    &property_title,
                    &property_location,
                    action_label,
                    &review_notes,
                    &action_url,
                    &header_asset_url(HEADER_REVIEW),
                );
                if let Err(error) = state.mail_service.send(email).await {
                    tracing::error!("failed to send property moderation email: {error:?}");
                }
            }
        }
    }

    Ok(Json(report))
}
