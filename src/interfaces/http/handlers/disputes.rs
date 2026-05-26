use axum::{Json, extract::{Path, State}, http::StatusCode};
use crate::interfaces::http::state::AppState;
// use crate::domain::disputes::repository as disputes_repo;
use crate::interfaces::http::middleware::auth::AuthUser;
use crate::domain::disputes::model::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateDisputeInput {
    pub booking_id: i32,
    pub property_id: i32,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct AddDisputeQuestionInput {
    pub party_id: i32,
    pub question: String,
}

#[derive(Debug, Deserialize)]
pub struct AddDisputeAnswerInput {
    pub question_id: i32,
    pub answer: String,
}

#[derive(Debug, Deserialize)]
pub struct SetDisputeVerdictInput {
    pub verdict: String,
}

// POST /api/v1/disputes
pub async fn create_dispute(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(input): Json<CreateDisputeInput>,
) -> Result<Json<Dispute>, StatusCode> {
    // TODO: Implement create_dispute logic or import correct repository
    Err(StatusCode::NOT_IMPLEMENTED)
}

// GET /api/v1/disputes
pub async fn list_disputes(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<Dispute>>, StatusCode> {
    // TODO: Implement get_disputes_for_user logic or import correct repository
    Err(StatusCode::NOT_IMPLEMENTED)
}

// GET /api/v1/disputes/:id
pub async fn get_dispute(
    State(state): State<AppState>,
    Path(dispute_id): Path<i32>,
) -> Result<Json<Dispute>, StatusCode> {
    // TODO: Implement get_dispute_by_id logic or import correct repository
    Err(StatusCode::NOT_IMPLEMENTED)
}

// POST /api/v1/disputes/:id/questions
pub async fn add_dispute_question(
    State(state): State<AppState>,
    Path(dispute_id): Path<i32>,
    Json(input): Json<AddDisputeQuestionInput>,
) -> Result<Json<DisputeQuestion>, StatusCode> {
    // TODO: Implement add_dispute_question logic or import correct repository
    Err(StatusCode::NOT_IMPLEMENTED)
}

// POST /api/v1/disputes/questions/:id/answer
pub async fn add_dispute_answer(
    State(state): State<AppState>,
    Path(question_id): Path<i32>,
    Json(input): Json<AddDisputeAnswerInput>,
) -> Result<Json<DisputeAnswer>, StatusCode> {
    // TODO: Implement add_dispute_answer logic or import correct repository
    Err(StatusCode::NOT_IMPLEMENTED)
}

// POST /api/v1/disputes/:id/verdict
pub async fn set_dispute_verdict(
    State(state): State<AppState>,
    Path(dispute_id): Path<i32>,
    Json(input): Json<SetDisputeVerdictInput>,
) -> Result<Json<Dispute>, StatusCode> {
    // TODO: Implement set_dispute_verdict logic or import correct repository
    Err(StatusCode::NOT_IMPLEMENTED)
}
