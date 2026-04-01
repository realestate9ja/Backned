use crate::{
    domain::{
        properties::PropertyRepository,
        responses::ResponseRepository,
        trust::{
            CreateReportInput, CreateReviewInput, ModerateReportInput, Report, Review, ReviewView, TrustRepository,
        },
        users::{User, UserRepository},
        workflow::WorkflowRepository,
    },
    infrastructure::cache::CacheService,
    interfaces::http::errors::AppError,
    utils::validation,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct TrustService {
    trust: TrustRepository,
    users: UserRepository,
    responses: ResponseRepository,
    properties: PropertyRepository,
    workflow: WorkflowRepository,
    cache: CacheService,
}

impl TrustService {
    pub fn new(
        trust: TrustRepository,
        users: UserRepository,
        responses: ResponseRepository,
        properties: PropertyRepository,
        workflow: WorkflowRepository,
        cache: CacheService,
    ) -> Self {
        Self {
            trust,
            users,
            responses,
            properties,
            workflow,
            cache,
        }
    }

    pub async fn create_review(&self, actor: &User, input: CreateReviewInput) -> Result<Review, AppError> {
        if input.rating < 1 || input.rating > 5 {
            return Err(AppError::bad_request("rating must be between 1 and 5"));
        }
        validation::validate_required(&input.comment, "comment")?;
        self.users
            .find_by_id(input.reviewee_id)
            .await?
            .ok_or_else(|| AppError::not_found("reviewee not found"))?;
        if actor.id == input.reviewee_id {
            return Err(AppError::bad_request("you cannot review yourself"));
        }
        if let Some(response_id) = input.response_id {
            self.responses
                .find_context(response_id)
                .await?
                .ok_or_else(|| AppError::not_found("response not found"))?;
            let meaningful = self
                .workflow
                .has_meaningful_interaction(
                    response_id,
                    actor.id,
                    input.reviewee_id,
                    input.property_id,
                )
                .await?;
            if !meaningful {
                return Err(AppError::forbidden(
                    "reviews require a completed live session or certified site visit with the other party",
                ));
            }
        } else {
            return Err(AppError::bad_request("response_id is required to submit a review"));
        }

        let review = self
            .trust
            .create_review(
                actor.id,
                input.reviewee_id,
                input.property_id,
                input.response_id,
                input.rating,
                input.comment.trim(),
            )
            .await
            .map_err(AppError::from)?;

        if let Some(property_id) = review.property_id {
            let low_review_count = self.trust.count_low_reviews_for_property(property_id).await?;
            if low_review_count >= 3 {
                self.properties.suspend_property(property_id).await?;
                self.invalidate_property_cache().await?;
            }
        }

        Ok(review)
    }

    pub async fn list_reviews_for_user(&self, user_id: Uuid) -> Result<Vec<ReviewView>, AppError> {
        self.users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::not_found("user not found"))?;
        self.trust.list_reviews_for_user(user_id, 50).await.map_err(Into::into)
    }

    pub async fn create_report(&self, actor: &User, input: CreateReportInput) -> Result<Report, AppError> {
        if !matches!(input.violation_type.as_str(), "quality" | "fraud" | "other") {
            return Err(AppError::bad_request("invalid violation_type"));
        }
        validation::validate_required(&input.reason, "reason")?;
        validation::validate_required(&input.details, "details")?;
        if input.reported_user_id.is_none()
            && input.property_id.is_none()
            && input.post_id.is_none()
            && input.response_id.is_none()
        {
            return Err(AppError::bad_request("report must target a user, property, post, or response"));
        }
        if let Some(user_id) = input.reported_user_id {
            self.users
                .find_by_id(user_id)
                .await?
                .ok_or_else(|| AppError::not_found("reported user not found"))?;
        }
        if let Some(response_id) = input.response_id {
            self.responses
                .find_context(response_id)
                .await?
                .ok_or_else(|| AppError::not_found("response not found"))?;
        }

        self.trust
            .create_report(
                actor.id,
                input.reported_user_id,
                input.property_id,
                input.post_id,
                input.response_id,
                input.violation_type.trim(),
                input.reason.trim(),
                input.details.trim(),
            )
            .await
            .map_err(Into::into)
    }

    pub async fn moderate_report(&self, report_id: Uuid, input: ModerateReportInput) -> Result<Report, AppError> {
        if !matches!(input.status.as_str(), "upheld" | "dismissed") {
            return Err(AppError::bad_request("invalid moderation status"));
        }
        validation::validate_required(&input.review_notes, "review_notes")?;

        let existing = self
            .trust
            .find_report(report_id)
            .await?
            .ok_or_else(|| AppError::not_found("report not found"))?;

        let report = self
            .trust
            .moderate_report(report_id, input.status.trim(), input.review_notes.trim())
            .await?
            .ok_or_else(|| AppError::not_found("report not found"))?;

        if report.status == "upheld" {
            self.apply_upheld_report(&existing).await?;
        }

        Ok(report)
    }

    async fn apply_upheld_report(&self, report: &Report) -> Result<(), AppError> {
        match report.violation_type.as_str() {
            "quality" => {
                if let Some(user_id) = report.reported_user_id {
                    self.users.apply_quality_violation(user_id).await?;
                }
                if let Some(property_id) = report.property_id {
                    let count = self
                        .trust
                        .count_upheld_reports_for_property_by_type(property_id, "quality")
                        .await?;
                    if count >= 3 {
                        self.properties.suspend_property(property_id).await?;
                        self.invalidate_property_cache().await?;
                    }
                }
            }
            "fraud" => {
                if let Some(user_id) = report.reported_user_id {
                    self.users.apply_fraud_violation(user_id).await?;
                }
                if let Some(property_id) = report.property_id {
                    self.properties.suspend_property(property_id).await?;
                    self.invalidate_property_cache().await?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn invalidate_property_cache(&self) -> Result<(), AppError> {
        self.cache.invalidate_namespace("properties:list").await?;
        self.cache.invalidate_namespace("properties:detail").await?;
        Ok(())
    }
}
