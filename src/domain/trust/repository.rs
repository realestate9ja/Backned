use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::trust::{Report, Review, ReviewView, UserReviewSummary};

#[derive(Clone)]
pub struct TrustRepository {
    pool: PgPool,
}

impl TrustRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_review(
        &self,
        reviewer_id: Uuid,
        reviewee_id: Uuid,
        property_id: Option<Uuid>,
        response_id: Option<Uuid>,
        rating: i16,
        comment: &str,
    ) -> Result<Review> {
        let review = sqlx::query_as::<_, Review>(
            r#"
            INSERT INTO reviews (id, reviewer_id, reviewee_id, property_id, response_id, rating, comment)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, reviewer_id, reviewee_id, property_id, response_id, rating, comment, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(reviewer_id)
        .bind(reviewee_id)
        .bind(property_id)
        .bind(response_id)
        .bind(rating)
        .bind(comment)
        .fetch_one(&self.pool)
        .await?;

        Ok(review)
    }

    pub async fn list_reviews_for_user(&self, user_id: Uuid, limit: i64) -> Result<Vec<ReviewView>> {
        let reviews = sqlx::query_as::<_, ReviewView>(
            r#"
            SELECT
                r.id,
                r.reviewer_id,
                reviewer.full_name AS reviewer_name,
                r.reviewee_id,
                r.property_id,
                r.response_id,
                r.rating,
                r.comment,
                r.created_at
            FROM reviews r
            INNER JOIN users reviewer ON reviewer.id = r.reviewer_id
            WHERE r.reviewee_id = $1
            ORDER BY r.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(reviews)
    }

    pub async fn summary_for_user(&self, user_id: Uuid) -> Result<UserReviewSummary> {
        let summary = sqlx::query_as::<_, UserReviewSummary>(
            r#"
            SELECT
                $1::uuid AS user_id,
                AVG(rating)::double precision AS average_rating,
                COUNT(*)::bigint AS review_count
            FROM reviews
            WHERE reviewee_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(summary)
    }

    pub async fn create_report(
        &self,
        reporter_id: Uuid,
        reported_user_id: Option<Uuid>,
        property_id: Option<Uuid>,
        post_id: Option<Uuid>,
        response_id: Option<Uuid>,
        violation_type: &str,
        reason: &str,
        details: &str,
    ) -> Result<Report> {
        let report = sqlx::query_as::<_, Report>(
            r#"
            INSERT INTO reports (
                id, reporter_id, reported_user_id, property_id, post_id, response_id, violation_type, reason, details, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'open')
            RETURNING
                id, reporter_id, reported_user_id, property_id, post_id, response_id,
                violation_type, reason, details, status, reviewed_by, reviewed_at, review_notes, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(reporter_id)
        .bind(reported_user_id)
        .bind(property_id)
        .bind(post_id)
        .bind(response_id)
        .bind(violation_type)
        .bind(reason)
        .bind(details)
        .fetch_one(&self.pool)
        .await?;

        Ok(report)
    }

    pub async fn find_report(&self, report_id: Uuid) -> Result<Option<Report>> {
        let report = sqlx::query_as::<_, Report>(
            r#"
            SELECT
                id, reporter_id, reported_user_id, property_id, post_id, response_id,
                violation_type, reason, details, status, reviewed_by, reviewed_at, review_notes,
                created_at, updated_at
            FROM reports
            WHERE id = $1
            "#,
        )
        .bind(report_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(report)
    }

    pub async fn moderate_report(
        &self,
        report_id: Uuid,
        status: &str,
        review_notes: &str,
    ) -> Result<Option<Report>> {
        let report = sqlx::query_as::<_, Report>(
            r#"
            UPDATE reports
            SET status = $2,
                review_notes = $3,
                reviewed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, reporter_id, reported_user_id, property_id, post_id, response_id,
                violation_type, reason, details, status, reviewed_by, reviewed_at, review_notes,
                created_at, updated_at
            "#,
        )
        .bind(report_id)
        .bind(status)
        .bind(review_notes)
        .fetch_optional(&self.pool)
        .await?;

        Ok(report)
    }

    pub async fn count_low_reviews_for_property(&self, property_id: Uuid) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::bigint
            FROM reviews
            WHERE property_id = $1 AND rating <= 2
            "#,
        )
        .bind(property_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn count_upheld_reports_for_property_by_type(
        &self,
        property_id: Uuid,
        violation_type: &str,
    ) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::bigint
            FROM reports
            WHERE property_id = $1
              AND violation_type = $2
              AND status = 'upheld'
            "#,
        )
        .bind(property_id)
        .bind(violation_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}
