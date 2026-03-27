use crate::{
    domain::{
        posts::{CreatePostInput, PostListItem, PostQuery, PostRepository},
        responses::{CreateResponseInput, ResponseCreated, ResponseRepository},
        users::User,
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
    cache: CacheService,
}

impl PostService {
    pub fn new(posts: PostRepository, responses: ResponseRepository, cache: CacheService) -> Self {
        Self {
            posts,
            responses,
            cache,
        }
    }

    pub async fn create_post(&self, actor: &User, input: CreatePostInput) -> Result<Uuid, AppError> {
        validation::validate_money(input.budget, "budget")?;
        validation::validate_required(&input.location, "location")?;
        validation::validate_required(&input.description, "description")?;

        let post = self.posts.create(&input, actor.id).await?;
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
                    "page={}&per_page={}&location={}&min_budget={}&max_budget={}",
                    pagination.page(),
                    pagination.per_page(),
                    query.location.clone().unwrap_or_default(),
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
