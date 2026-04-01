use crate::{
    application::services::UserService,
    domain::{
        notifications::AgentPostNotificationItem,
        users::{
            AgentNotificationSettingsView, AgentProfile, DashboardResponse, UpdateAgentNotificationSettingsInput,
            UpdateAgentVerificationInput, User, UserPublicView,
        },
    },
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

    pub async fn update_agent_notification_settings(
        &self,
        actor: &User,
        input: UpdateAgentNotificationSettingsInput,
    ) -> Result<AgentNotificationSettingsView, AppError> {
        self.service.update_agent_notification_settings(actor, input).await
    }

    pub async fn list_agent_post_alerts(&self, actor: &User) -> Result<Vec<AgentPostNotificationItem>, AppError> {
        self.service.list_agent_post_alerts(actor).await
    }

    pub async fn get_dashboard(&self, actor: &User) -> Result<DashboardResponse, AppError> {
        self.service.get_dashboard(actor).await
    }

    pub async fn update_agent_verification(
        &self,
        actor: &User,
        agent_id: Uuid,
        input: UpdateAgentVerificationInput,
    ) -> Result<UserPublicView, AppError> {
        self.service.update_agent_verification(actor, agent_id, input).await
    }
}
