use crate::{
    domain::{
        notifications::{AgentPostNotificationItem, NotificationRepository},
        posts::PostRepository,
        properties::PropertyRepository,
        users::{
            AgentNotificationSettingsView, AgentProfile, DashboardResponse, LandlordDashboard, UpdateAgentNotificationSettingsInput,
            User, UserPublicView, UserRepository, UserRole, AgentDashboard, BuyerDashboard,
        },
    },
    infrastructure::cache::CacheService,
    interfaces::http::errors::AppError,
    utils::{pagination::Pagination, validation},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    users: UserRepository,
    properties: PropertyRepository,
    posts: PostRepository,
    notifications: NotificationRepository,
    cache: CacheService,
}

impl UserService {
    pub fn new(
        users: UserRepository,
        properties: PropertyRepository,
        posts: PostRepository,
        notifications: NotificationRepository,
        cache: CacheService,
    ) -> Self {
        Self {
            users,
            properties,
            posts,
            notifications,
            cache,
        }
    }

    pub async fn get_user(&self, id: Uuid) -> Result<UserPublicView, AppError> {
        let user = self
            .users
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("user not found"))?;

        Ok(UserPublicView::from(user))
    }

    pub async fn list_agents(&self, pagination: Pagination) -> Result<Vec<AgentProfile>, AppError> {
        let cache_key = self
            .cache
            .versioned_key(
                "agents",
                &format!("page={}&per_page={}", pagination.page(), pagination.per_page()),
            )
            .await?;
        if let Some(cached) = self.cache.get_json::<Vec<AgentProfile>>(&cache_key).await? {
            return Ok(cached);
        }

        let items = self
            .users
            .list_agents(pagination.limit(), pagination.offset())
            .await?;
        self.cache.set_json(&cache_key, &items).await?;

        Ok(items)
    }

    pub async fn update_agent_notification_settings(
        &self,
        actor: &User,
        input: UpdateAgentNotificationSettingsInput,
    ) -> Result<AgentNotificationSettingsView, AppError> {
        if actor.role != UserRole::Agent {
            return Err(AppError::forbidden("only agents can update notification settings"));
        }

        if input.notifications_enabled {
            let city = input
                .operating_city
                .as_deref()
                .ok_or_else(|| AppError::bad_request("operating_city is required when notifications are enabled"))?;
            let state = input
                .operating_state
                .as_deref()
                .ok_or_else(|| AppError::bad_request("operating_state is required when notifications are enabled"))?;
            validation::validate_required(city, "operating_city")?;
            validation::validate_required(state, "operating_state")?;
        }

        let updated = self
            .users
            .update_agent_notification_settings(actor.id, &input)
            .await?;
        self.cache.invalidate_namespace("agents").await?;

        Ok(AgentNotificationSettingsView {
            notifications_enabled: updated.notifications_enabled,
            operating_city: updated.operating_city,
            operating_state: updated.operating_state,
        })
    }

    pub async fn list_agent_post_alerts(&self, actor: &User) -> Result<Vec<AgentPostNotificationItem>, AppError> {
        if actor.role != UserRole::Agent {
            return Err(AppError::forbidden("only agents can view post alerts"));
        }

        Ok(self.notifications.list_for_agent(actor.id, 20).await?)
    }

    pub async fn get_dashboard(&self, actor: &User) -> Result<DashboardResponse, AppError> {
        let profile = UserPublicView::from(actor.clone());

        let response = match actor.role {
            UserRole::Buyer => {
                let recent_posts = self.posts.list_recent_by_author(actor.id, 10).await?;
                let my_posts_count = self.posts.count_by_author(actor.id).await?;
                DashboardResponse {
                    role: actor.role,
                    profile,
                    buyer: Some(BuyerDashboard {
                        my_posts_count,
                        recent_posts,
                    }),
                    agent: None,
                    landlord: None,
                }
            }
            UserRole::Agent => {
                let recent_properties = self.properties.list_recent_managed_by_agent(actor.id, 10).await?;
                let recent_post_alerts = self.notifications.list_for_agent(actor.id, 10).await?;
                let managed_properties_count = self.properties.count_managed_by_agent(actor.id).await?;
                let service_apartments_count = self
                    .properties
                    .count_service_apartments_managed_by_agent(actor.id)
                    .await?;
                let unread_post_alerts_count = self.notifications.count_unread_for_agent(actor.id).await?;

                DashboardResponse {
                    role: actor.role,
                    profile,
                    buyer: None,
                    agent: Some(AgentDashboard {
                        managed_properties_count,
                        service_apartments_count,
                        unread_post_alerts_count,
                        recent_properties,
                        recent_post_alerts,
                    }),
                    landlord: None,
                }
            }
            UserRole::Landlord => {
                let recent_properties = self.properties.list_recent_by_owner(actor.id, 10).await?;
                let owned_properties_count = self.properties.count_owned_by_user(actor.id).await?;
                let assigned_agents_count = self
                    .properties
                    .count_distinct_assigned_agents_for_owner(actor.id)
                    .await?;

                DashboardResponse {
                    role: actor.role,
                    profile,
                    buyer: None,
                    agent: None,
                    landlord: Some(LandlordDashboard {
                        owned_properties_count,
                        assigned_agents_count,
                        recent_properties,
                    }),
                }
            }
        };

        Ok(response)
    }
}
