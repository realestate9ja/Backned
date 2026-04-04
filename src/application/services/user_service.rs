use crate::{
    domain::{
        notifications::{AgentPostNotificationItem, NotificationRepository},
        posts::PostRepository,
        properties::PropertyRepository,
        responses::{ResponseRepository, SeekerActiveRequest},
        trust::TrustRepository,
        users::{
            AgentDashboard, AgentNotificationSettingsView, AgentProfile, DashboardResponse,
            LandlordDashboard, UpdateAgentNotificationSettingsInput, UpdateAgentVerificationInput, User,
            UserPublicView, UserRepository, UserRole, SeekerDashboard,
        },
        workflow::WorkflowRepository,
    },
    infrastructure::{cache::CacheService, email::MailService},
    interfaces::http::errors::AppError,
    utils::{pagination::Pagination, validation},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    users: UserRepository,
    properties: PropertyRepository,
    posts: PostRepository,
    responses: ResponseRepository,
    notifications: NotificationRepository,
    workflow: WorkflowRepository,
    trust: TrustRepository,
    cache: CacheService,
    mail_service: MailService,
}

impl UserService {
    pub fn new(
        users: UserRepository,
        properties: PropertyRepository,
        posts: PostRepository,
        responses: ResponseRepository,
        notifications: NotificationRepository,
        workflow: WorkflowRepository,
        trust: TrustRepository,
        cache: CacheService,
        mail_service: MailService,
    ) -> Self {
        Self {
            users,
            properties,
            posts,
            responses,
            notifications,
            workflow,
            trust,
            cache,
            mail_service,
        }
    }

    pub async fn get_user(&self, id: Uuid) -> Result<UserPublicView, AppError> {
        let user = self
            .users
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("user not found"))?;
        let summary = self.trust.summary_for_user(user.id).await?;
        let mut view = UserPublicView::from(user);
        view.average_rating = summary.average_rating;
        view.review_count = summary.review_count;
        Ok(view)
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

    pub async fn update_agent_verification(
        &self,
        actor: &User,
        agent_id: Uuid,
        input: UpdateAgentVerificationInput,
    ) -> Result<UserPublicView, AppError> {
        if !actor.role.can_moderate() {
            return Err(AppError::forbidden("only admins can verify agents"));
        }
        if !matches!(input.verification_status.as_str(), "pending" | "verified" | "rejected") {
            return Err(AppError::bad_request("invalid verification_status"));
        }

        let updated = self
            .users
            .update_agent_verification(
                agent_id,
                input.verification_status.trim(),
                input.verification_notes.as_deref().map(str::trim),
            )
            .await?
            .ok_or_else(|| AppError::not_found("agent not found"))?;
        self.cache.invalidate_namespace("agents").await?;
        let email = self.mail_service.kyc_status_email(
            updated.email.clone(),
            &updated.full_name,
            &updated.verification_status,
            updated.verification_notes.as_deref(),
        );
        self.mail_service.send(email).await?;

        let summary = self.trust.summary_for_user(updated.id).await?;
        let mut view = UserPublicView::from(updated);
        view.average_rating = summary.average_rating;
        view.review_count = summary.review_count;
        Ok(view)
    }

    pub async fn get_dashboard(&self, actor: &User) -> Result<DashboardResponse, AppError> {
        let summary = self.trust.summary_for_user(actor.id).await?;
        let mut profile = UserPublicView::from(actor.clone());
        profile.average_rating = summary.average_rating;
        profile.review_count = summary.review_count;

        let response = match actor.role {
            UserRole::Seeker => {
                let active_requests = self.posts.list_active_by_author(actor.id, 10).await?;
                let active_requests = futures_from_requests(&self.responses, active_requests).await?;
                let live_video_sessions = self.workflow.list_live_video_sessions_for_user(actor.id, 20).await?;
                let site_visits = self.workflow.list_site_visit_views_for_user(actor.id, 20).await?;
                DashboardResponse {
                    role: actor.role,
                    profile,
                    seeker: Some(SeekerDashboard {
                        active_requests,
                        live_video_sessions,
                        site_visits,
                    }),
                    agent: None,
                    landlord: None,
                }
            }
            UserRole::Agent => {
                let managed_properties = self.properties.list_recent_managed_by_agent(actor.id, 20).await?;
                let service_apartments = self
                    .properties
                    .list_service_apartments_managed_by_agent(actor.id, 20)
                    .await?;
                let unread_post_alerts = self.notifications.list_unread_for_agent(actor.id, 20).await?;
                let request_threads = self.workflow.list_threads_for_user(actor.id, 20).await?;
                let live_video_sessions = self.workflow.list_live_video_sessions_for_user(actor.id, 20).await?;
                let site_visits = self.workflow.list_site_visit_views_for_user(actor.id, 20).await?;

                DashboardResponse {
                    role: actor.role,
                    profile,
                    seeker: None,
                    agent: Some(AgentDashboard {
                        managed_properties,
                        service_apartments,
                        unread_post_alerts,
                        request_threads,
                        live_video_sessions,
                        site_visits,
                    }),
                    landlord: None,
                }
            }
            UserRole::Landlord => {
                let owned_properties = self.properties.list_recent_by_owner(actor.id, 20).await?;
                let pending_verification_properties = self
                    .properties
                    .list_recent_by_owner_and_status(
                        actor.id,
                        crate::domain::properties::PropertyStatus::PendingVerification,
                        20,
                    )
                    .await?;
                let agent_requests = self
                    .properties
                    .list_recent_by_owner(actor.id, 50)
                    .await?
                    .into_iter()
                    .filter(|property| !property.self_managed)
                    .map(|property| property.id)
                    .collect::<Vec<_>>();
                let mut request_items = Vec::new();
                for property_id in agent_requests {
                    if let Some(request) = self.workflow.find_property_agent_request(property_id).await? {
                        request_items.push(request);
                    }
                }

                DashboardResponse {
                    role: actor.role,
                    profile,
                    seeker: None,
                    agent: None,
                    landlord: Some(LandlordDashboard {
                        owned_properties,
                        pending_verification_properties,
                        agent_requests: request_items,
                    }),
                }
            }
            UserRole::Admin => DashboardResponse {
                role: actor.role,
                profile,
                seeker: None,
                agent: None,
                landlord: None,
            },
        };

        Ok(response)
    }
}

async fn futures_from_requests(
    responses: &ResponseRepository,
    requests: Vec<crate::domain::posts::PostListItem>,
) -> Result<Vec<SeekerActiveRequest>, AppError> {
    let mut items = Vec::with_capacity(requests.len());
    for request in requests {
        let request_id = request.id;
        let request_responses = responses.list_with_properties_for_post(request_id).await?;
        items.push(SeekerActiveRequest {
            request,
            responses: request_responses,
        });
    }
    Ok(items)
}
