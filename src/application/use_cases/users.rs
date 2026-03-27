use crate::{
    application::services::UserService,
    domain::users::{AgentProfile, UserPublicView},
    interfaces::http::errors::AppError,
    utils::pagination::Pagination,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct UserUseCases {
    service: UserService,
}

impl UserUseCases {
    pub fn new(service: UserService) -> Self {
        Self { service }
    }

    pub async fn get_user(&self, id: Uuid) -> Result<UserPublicView, AppError> {
        self.service.get_user(id).await
    }

    pub async fn list_agents(&self, pagination: Pagination) -> Result<Vec<AgentProfile>, AppError> {
        self.service.list_agents(pagination).await
    }
}

