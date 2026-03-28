use crate::{
    domain::{
        notifications::{AgentNotificationTarget, NotificationRepository},
        posts::{CreatePostInput, PostListItem, PostQuery, PostRepository},
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
    notifications: NotificationRepository,
    cache: CacheService,
}

impl PostService {
    pub fn new(
        posts: PostRepository,
        responses: ResponseRepository,
        users: UserRepository,
        notifications: NotificationRepository,
        cache: CacheService,
    ) -> Self {
        Self {
            posts,
            responses,
            users,
            notifications,
            cache,
        }
    }

    pub async fn create_post(&self, actor: &User, input: CreatePostInput) -> Result<Uuid, AppError> {
        validation::validate_money(input.budget, "budget")?;
        validation::validate_required(&input.location, "location")?;
        validation::validate_required(&input.city, "city")?;
        validation::validate_required(&input.state, "state")?;
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
                    "page={}&per_page={}&location={}&city={}&state={}&min_budget={}&max_budget={}",
                    pagination.page(),
                    pagination.per_page(),
                    query.location.clone().unwrap_or_default(),
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

        if !self.posts.exists(post_id).await? {
            return Err(AppError::not_found("post not found"));
        }

        let response = self.responses.create(post_id, actor.id, &input).await?;
        self.cache.invalidate_namespace("posts:list").await?;
        Ok(response.into())
    }
}
