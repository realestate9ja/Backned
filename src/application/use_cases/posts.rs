use crate::{
    application::services::PostService,
    domain::{
        posts::{CreatePostInput, PostListItem, PostQuery},
        responses::{CreateResponseInput, ResponseCreated},
        users::User,
    },
    interfaces::http::errors::AppError,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostUseCases {
    service: PostService,
}

impl PostUseCases {
    pub fn new(service: PostService) -> Self {
        Self { service }
    }

    pub async fn create_post(&self, actor: &User, input: CreatePostInput) -> Result<Uuid, AppError> {
        self.service.create_post(actor, input).await
    }

    pub async fn list_posts(&self, query: PostQuery) -> Result<Vec<PostListItem>, AppError> {
        self.service.list_posts(query).await
    }

    pub async fn respond(
        &self,
        actor: &User,
        post_id: Uuid,
        input: CreateResponseInput,
    ) -> Result<ResponseCreated, AppError> {
        self.service.respond(actor, post_id, input).await
    }
}

