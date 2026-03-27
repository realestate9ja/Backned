use crate::{
    application::services::PropertyService,
    domain::{
        properties::{CreatePropertyInput, PropertyDetail, PropertyListItem, PropertyQuery},
        users::User,
    },
    interfaces::http::errors::AppError,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct PropertyUseCases {
    service: PropertyService,
}

impl PropertyUseCases {
    pub fn new(service: PropertyService) -> Self {
        Self { service }
    }

    pub async fn create(
        &self,
        actor: &User,
        input: CreatePropertyInput,
    ) -> Result<PropertyDetail, AppError> {
        self.service.create(actor, input).await
    }

    pub async fn list(&self, query: PropertyQuery) -> Result<Vec<PropertyListItem>, AppError> {
        self.service.list(query).await
    }

    pub async fn get_by_id(
        &self,
        id: Uuid,
        actor: Option<&User>,
    ) -> Result<PropertyDetail, AppError> {
        self.service.get_by_id(id, actor).await
    }
}

