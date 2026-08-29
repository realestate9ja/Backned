use crate::domain::disputes::model::*;
use sqlx::FromRow;
use sqlx::{Pool, Postgres};

// Fetch disputes for a user (seeker/agent or admin)
pub async fn get_disputes_for_user(pool: &Pool<Postgres>, user_id: i32, role: &str) -> Result<Vec<Dispute>, sqlx::Error> {
    let disputes = sqlx::query_as::<_, Dispute>(
        "SELECT * FROM disputes WHERE id IN (SELECT dispute_id FROM dispute_parties WHERE user_id = $1 AND role = $2) ORDER BY created_at DESC"
    )
    .bind(user_id)
    .bind(role)
    .fetch_all(pool)
    .await?;
    Ok(disputes)
}

// Fetch a single dispute by id (with parties)
pub async fn get_dispute_by_id(pool: &Pool<Postgres>, dispute_id: i32) -> Result<Dispute, sqlx::Error> {
    let dispute = sqlx::query_as::<_, Dispute>("SELECT * FROM disputes WHERE id = $1")
        .bind(dispute_id)
        .fetch_one(pool)
        .await?;
    Ok(dispute)
}

// Create a new dispute
pub async fn create_dispute(
    pool: &Pool<Postgres>,
    booking_id: i32,
    property_id: i32,
    raiser_id: Option<i32>,
    raiser_role: &str,
    reason: &str,
) -> Result<Dispute, sqlx::Error> {
    let rec = sqlx::query_as::<_, Dispute>(
        "INSERT INTO disputes (booking_id, property_id, raiser_id, raiser_role, reason, status) VALUES ($1, $2, $3, $4, $5, 'open') RETURNING *"
    )
    .bind(booking_id)
    .bind(property_id)
    .bind(raiser_id)
    .bind(raiser_role)
    .bind(reason)
    .fetch_one(pool)
    .await?;
    Ok(rec)
}

// Add a party to a dispute
pub async fn add_dispute_party(
    pool: &Pool<Postgres>,
    dispute_id: i32,
    user_id: i32,
    role: &str,
) -> Result<DisputeParty, sqlx::Error> {
    let rec = sqlx::query_as::<_, DisputeParty>(
        "INSERT INTO dispute_parties (dispute_id, user_id, role) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(dispute_id)
    .bind(user_id)
    .bind(role)
    .fetch_one(pool)
    .await?;
    Ok(rec)
}

// Add a question (admin to party)
pub async fn add_dispute_question(
    pool: &Pool<Postgres>,
    dispute_id: i32,
    party_id: i32,
    question: &str,
) -> Result<DisputeQuestion, sqlx::Error> {
    let rec = sqlx::query_as::<_, DisputeQuestion>(
        "INSERT INTO dispute_questions (dispute_id, party_id, question, asked_by_admin) VALUES ($1, $2, $3, TRUE) RETURNING *"
    )
    .bind(dispute_id)
    .bind(party_id)
    .bind(question)
    .fetch_one(pool)
    .await?;
    Ok(rec)
}

// Add an answer (party to admin)
pub async fn add_dispute_answer(
    pool: &Pool<Postgres>,
    question_id: i32,
    answer: &str,
) -> Result<DisputeAnswer, sqlx::Error> {
    let rec = sqlx::query_as::<_, DisputeAnswer>(
        "INSERT INTO dispute_answers (question_id, answer, answered_by_party) VALUES ($1, $2, TRUE) RETURNING *"
    )
    .bind(question_id)
    .bind(answer)
    .fetch_one(pool)
    .await?;
    Ok(rec)
}

// Set verdict
pub async fn set_dispute_verdict(
    pool: &Pool<Postgres>,
    dispute_id: i32,
    verdict: &str,
) -> Result<Dispute, sqlx::Error> {
    let rec = sqlx::query_as::<_, Dispute>(
        "UPDATE disputes SET verdict = $1, verdict_at = NOW(), status = 'closed', updated_at = NOW() WHERE id = $2 RETURNING *"
    )
    .bind(verdict)
    .bind(dispute_id)
    .fetch_one(pool)
    .await?;
    Ok(rec)
}
