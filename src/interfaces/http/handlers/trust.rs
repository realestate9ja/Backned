use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    domain::trust::{CreateReportInput, CreateReviewInput, ModerateReportInput},
    interfaces::http::{
        errors::AppError,
        middleware::auth::AuthUser,
        state::AppState,
    },
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
    let report = state.trust_use_cases.moderate_report(report_id, payload).await?;
    Ok(Json(report))
}
