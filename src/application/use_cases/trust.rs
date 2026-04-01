use crate::{
    application::services::TrustService,
    domain::{
        trust::{CreateReportInput, CreateReviewInput, ModerateReportInput, Report, Review, ReviewView},
        users::User,
    },
    interfaces::http::errors::AppError,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct TrustUseCases {
    service: TrustService,
}

impl TrustUseCases {
    pub fn new(service: TrustService) -> Self {
        Self { service }
    }

    pub async fn create_review(&self, actor: &User, input: CreateReviewInput) -> Result<Review, AppError> {
        self.service.create_review(actor, input).await
    }

    pub async fn list_reviews_for_user(&self, user_id: Uuid) -> Result<Vec<ReviewView>, AppError> {
        self.service.list_reviews_for_user(user_id).await
    }

    pub async fn create_report(&self, actor: &User, input: CreateReportInput) -> Result<Report, AppError> {
        self.service.create_report(actor, input).await
    }

    pub async fn moderate_report(&self, report_id: Uuid, input: ModerateReportInput) -> Result<Report, AppError> {
        self.service.moderate_report(report_id, input).await
    }
}
