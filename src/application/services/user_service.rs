use crate::{
    domain::users::{AgentProfile, UserPublicView, UserRepository},
    infrastructure::cache::CacheService,
    interfaces::http::errors::AppError,
    utils::pagination::Pagination,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    users: UserRepository,
    cache: CacheService,
}

impl UserService {
    pub fn new(users: UserRepository, cache: CacheService) -> Self {
        Self { users, cache }
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
}
