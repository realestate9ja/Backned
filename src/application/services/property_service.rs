use crate::{
    domain::{
        properties::{CreatePropertyInput, PropertyDetail, PropertyListItem, PropertyQuery, PropertyRepository},
        users::{User, UserRepository, UserRole},
    },
    infrastructure::cache::CacheService,
    interfaces::http::errors::AppError,
    utils::{pagination::Pagination, validation},
};

#[derive(Clone)]
pub struct PropertyService {
    properties: PropertyRepository,
    users: UserRepository,
    cache: CacheService,
}

impl PropertyService {
    pub fn new(properties: PropertyRepository, users: UserRepository, cache: CacheService) -> Self {
        Self {
            properties,
            users,
            cache,
        }
    }

    pub async fn create(
        &self,
        actor: &User,
        input: CreatePropertyInput,
    ) -> Result<PropertyDetail, AppError> {
        if !actor.role.can_manage_properties() {
            return Err(AppError::forbidden("only agents and landlords can create properties"));
        }

        validation::validate_required(&input.title, "title")?;
        validation::validate_money(input.price, "price")?;
        validation::validate_required(&input.location, "location")?;
        validation::validate_required(&input.exact_address, "exact_address")?;
        validation::validate_required(&input.description, "description")?;
        validation::validate_required(&input.contact_name, "contact_name")?;
        validation::validate_required(&input.contact_phone, "contact_phone")?;
        validation::validate_non_empty_vec(&input.images, "images")?;

        let assigned_agent_id = match actor.role {
            UserRole::Agent => Some(actor.id),
            UserRole::Landlord => {
                if let Some(agent_id) = input.agent_id {
                    let agent = self
                        .users
                        .find_agent_by_id(agent_id)
                        .await?
                        .ok_or_else(|| AppError::bad_request("assigned agent does not exist"))?;
                    Some(agent.id)
                } else {
                    None
                }
            }
            UserRole::Buyer => None,
        };

        let property = self
            .properties
            .create(&input, actor.id, assigned_agent_id)
            .await?;
        self.cache.invalidate_namespace("properties:list").await?;
        self.cache.invalidate_namespace("properties:detail").await?;

        let detail = self
            .properties
            .find_detail_by_id(property.id)
            .await?
            .ok_or_else(|| AppError::internal("created property could not be loaded"))?;

        Ok(detail.sanitize_for_role(actor.role))
    }

    pub async fn list(
        &self,
        query: PropertyQuery,
    ) -> Result<Vec<PropertyListItem>, AppError> {
        let pagination = Pagination::new(query.page, query.per_page)?;
        let cache_key = self
            .cache
            .versioned_key(
                "properties:list",
                &format!(
                    "page={}&per_page={}&location={}&min_price={}&max_price={}",
                    pagination.page(),
                    pagination.per_page(),
                    query.location.clone().unwrap_or_default(),
                    query.min_price.map(|v| v.to_string()).unwrap_or_default(),
                    query.max_price.map(|v| v.to_string()).unwrap_or_default(),
                ),
            )
            .await?;
        if let Some(cached) = self.cache.get_json::<Vec<PropertyListItem>>(&cache_key).await? {
            return Ok(cached);
        }

        let items = self
            .properties
            .list(
                pagination.limit(),
                pagination.offset(),
                query.location.as_deref(),
                query.min_price,
                query.max_price,
            )
            .await?;
        self.cache.set_json(&cache_key, &items).await?;

        Ok(items)
    }

    pub async fn get_by_id(&self, id: uuid::Uuid, actor: Option<&User>) -> Result<PropertyDetail, AppError> {
        let visibility = match actor.map(|user| user.role) {
            Some(UserRole::Agent | UserRole::Landlord) => "privileged",
            _ => "restricted",
        };
        let cache_key = self
            .cache
            .versioned_key("properties:detail", &format!("{id}:{visibility}"))
            .await?;
        if let Some(cached) = self.cache.get_json::<PropertyDetail>(&cache_key).await? {
            return Ok(cached);
        }

        let detail = self
            .properties
            .find_detail_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("property not found"))?;

        let role = actor.map(|user| user.role).unwrap_or(UserRole::Buyer);
        let detail = detail.sanitize_for_role(role);
        self.cache.set_json(&cache_key, &detail).await?;
        Ok(detail)
    }
}
