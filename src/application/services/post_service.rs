use crate::{
    domain::{
        notifications::{AgentNotificationTarget, NotificationRepository},
        posts::{CreatePostInput, PostListItem, PostQuery, PostRepository},
        properties::PropertyRepository,
        responses::{CreateResponseInput, ResponseCreated, ResponseRepository},
        users::{User, UserRepository},
    },
    infrastructure::cache::CacheService,
    interfaces::http::errors::AppError,
    utils::{pagination::Pagination, validation},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostService {
    posts: PostRepository,
    responses: ResponseRepository,
    users: UserRepository,
    properties: PropertyRepository,
    notifications: NotificationRepository,
    cache: CacheService,
}

impl PostService {
    pub fn new(
        posts: PostRepository,
        responses: ResponseRepository,
        users: UserRepository,
        properties: PropertyRepository,
        notifications: NotificationRepository,
        cache: CacheService,
    ) -> Self {
        Self {
            posts,
            responses,
            users,
            properties,
            notifications,
            cache,
        }
    }

    pub async fn create_post(&self, actor: &User, input: CreatePostInput) -> Result<Uuid, AppError> {
        validation::validate_required(&input.request_title, "request_title")?;
        validation::validate_required(&input.area, "area")?;
        validation::validate_required(&input.city, "city")?;
        validation::validate_required(&input.state, "state")?;
        validation::validate_required(&input.property_type, "property_type")?;
        validation::validate_required(&input.pricing_preference, "pricing_preference")?;
        validation::validate_money(input.min_budget, "min_budget")?;
        validation::validate_money(input.max_budget, "max_budget")?;
        if input.max_budget < input.min_budget {
            return Err(AppError::bad_request("max_budget must be greater than or equal to min_budget"));
        }
        if input.bedrooms < 0 {
            return Err(AppError::bad_request("bedrooms must be greater than or equal to 0"));
        }
        validation::validate_non_empty_vec(&input.desired_features, "desired_features")?;
        validation::validate_required(&input.description, "description")?;

        let post = self.posts.create(&input, actor.id).await?;
        let recipients = self
            .users
            .list_notifiable_agents(&input.city, &input.state)
            .await?
            .into_iter()
            .map(|agent| AgentNotificationTarget {
                agent_id: agent.id,
                matched_city: agent.operating_city,
                matched_state: agent.operating_state,
            })
            .collect::<Vec<_>>();
        self.notifications.create_for_post(post.id, &recipients).await?;
        self.cache.invalidate_namespace("posts:list").await?;
        Ok(post.id)
    }

    pub async fn list_posts(&self, query: PostQuery) -> Result<Vec<PostListItem>, AppError> {
        let pagination = Pagination::new(query.page, query.per_page)?;
        let cache_key = self
            .cache
            .versioned_key(
                "posts:list",
                &format!(
                    "page={}&per_page={}&location={}&property_type={}&city={}&state={}&min_budget={}&max_budget={}",
                    pagination.page(),
                    pagination.per_page(),
                    query.location.clone().unwrap_or_default(),
                    query.property_type.clone().unwrap_or_default(),
                    query.city.clone().unwrap_or_default(),
                    query.state.clone().unwrap_or_default(),
                    query.min_budget.map(|v| v.to_string()).unwrap_or_default(),
                    query.max_budget.map(|v| v.to_string()).unwrap_or_default(),
                ),
            )
            .await?;
        if let Some(cached) = self.cache.get_json::<Vec<PostListItem>>(&cache_key).await? {
            return Ok(cached);
        }

        let items = self
            .posts
            .list(
                pagination.limit(),
                pagination.offset(),
                query.location.as_deref(),
                query.property_type.as_deref(),
                query.city.as_deref(),
                query.state.as_deref(),
                query.min_budget,
                query.max_budget,
            )
            .await?;
        self.cache.set_json(&cache_key, &items).await?;

        Ok(items)
    }

    pub async fn respond(
        &self,
        actor: &User,
        post_id: Uuid,
        input: CreateResponseInput,
    ) -> Result<ResponseCreated, AppError> {
        validation::validate_required(&input.message, "message")?;
        if !actor.role.can_manage_properties() {
            return Err(AppError::forbidden(
                "only agents and landlords can respond to requests with listed properties",
            ));
        }
        if input.property_ids.is_empty() {
            return Err(AppError::bad_request("property_ids must not be empty"));
        }

        if !self.posts.exists(post_id).await? {
            return Err(AppError::not_found("post not found"));
        }

        let properties = self
            .properties
            .list_owned_or_managed_by_user_for_ids(actor.id, &input.property_ids)
            .await?;
        if properties.len() != input.property_ids.len() {
            return Err(AppError::forbidden(
                "you can only respond with properties you own or manage on the platform",
            ));
        }

        let response = self.responses.create(post_id, actor.id, &input).await?;
        self.cache.invalidate_namespace("posts:list").await?;
        Ok(response)
    }
}
