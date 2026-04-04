use crate::{
    domain::{
        properties::{CreatePropertyInput, PropertyDetail, PropertyListItem, PropertyQuery, PropertyRepository},
        users::{User, UserRepository, UserRole},
        workflow::WorkflowRepository,
    },
    infrastructure::cache::CacheService,
    interfaces::http::errors::AppError,
    utils::{pagination::Pagination, validation},
};
use chrono::Utc;

#[derive(Clone)]
pub struct PropertyService {
    properties: PropertyRepository,
    users: UserRepository,
    workflow: WorkflowRepository,
    cache: CacheService,
}

impl PropertyService {
    pub fn new(
        properties: PropertyRepository,
        users: UserRepository,
        workflow: WorkflowRepository,
        cache: CacheService,
    ) -> Self {
        Self {
            properties,
            users,
            workflow,
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
        if actor.is_banned {
            return Err(AppError::forbidden("account is banned from listing properties"));
        }
        if actor
            .listing_restricted_until
            .is_some_and(|until| until > Utc::now())
        {
            return Err(AppError::forbidden("account is temporarily restricted from listing properties"));
        }
        if actor.role == UserRole::Agent && actor.verification_status != "verified" {
            return Err(AppError::forbidden("agent must be verified before listing properties"));
        }

        validation::validate_required(&input.title, "title")?;
        validation::validate_money(input.price, "price")?;
        validation::validate_required(&input.location, "location")?;
        validation::validate_required(&input.exact_address, "exact_address")?;
        validation::validate_required(&input.description, "description")?;
        validation::validate_required(&input.contact_name, "contact_name")?;
        validation::validate_required(&input.contact_phone, "contact_phone")?;
        validation::validate_non_empty_vec(&input.images, "images")?;

        let (assigned_agent_id, self_managed, requested_agent_id, status) = match actor.role {
            UserRole::Agent => (
                Some(actor.id),
                false,
                None,
                crate::domain::properties::PropertyStatus::Published,
            ),
            UserRole::Landlord => {
                let requested_agent_id = input.requested_agent_id;
                let self_managed = input.self_managed.unwrap_or(requested_agent_id.is_none());
                if self_managed && requested_agent_id.is_some() {
                    return Err(AppError::bad_request(
                        "self-managed properties cannot also request an agent",
                    ));
                }
                if let Some(agent_id) = requested_agent_id {
                    let agent = self
                        .users
                        .find_agent_by_id(agent_id)
                        .await?
                        .ok_or_else(|| AppError::bad_request("requested agent does not exist"))?;
                    (
                        None,
                        false,
                        Some(agent.id),
                        crate::domain::properties::PropertyStatus::PendingVerification,
                    )
                } else {
                    (
                        None,
                        true,
                        None,
                        crate::domain::properties::PropertyStatus::PendingVerification,
                    )
                }
            }
            UserRole::Seeker => (None, false, None, crate::domain::properties::PropertyStatus::Draft),
            UserRole::Admin => (None, false, None, crate::domain::properties::PropertyStatus::Draft),
        };

        let property = self
            .properties
            .create(
                &input,
                actor.id,
                assigned_agent_id,
                self_managed,
                status,
            )
            .await?;
        if actor.role == UserRole::Landlord && requested_agent_id.is_some() {
            self.workflow
                .create_property_agent_request(property.id, actor.id, requested_agent_id, None)
                .await?;
        }
        self.cache.invalidate_namespace("properties:list").await?;
        self.cache.invalidate_namespace("properties:detail").await?;

        let detail = self
            .properties
            .find_detail_by_id_including_unpublished(property.id)
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
        let is_related = |detail: &PropertyDetail, user: &User| detail.owner_id == user.id || detail.agent_id == Some(user.id);

        if let Some(user) = actor {
            let detail = self
                .properties
                .find_detail_by_id_including_unpublished(id)
                .await?
                .ok_or_else(|| AppError::not_found("property not found"))?;

            if detail.status != crate::domain::properties::PropertyStatus::Published && !is_related(&detail, user) {
                return Err(AppError::not_found("property not found"));
            }

            if is_related(&detail, user) {
                return Ok(detail);
            }

            let visibility = match user.role {
                UserRole::Agent | UserRole::Landlord | UserRole::Admin => "privileged",
                UserRole::Seeker => "restricted",
            };
            let cache_key = self
                .cache
                .versioned_key("properties:detail", &format!("{id}:{visibility}"))
                .await?;
            if detail.status == crate::domain::properties::PropertyStatus::Published {
                if let Some(cached) = self.cache.get_json::<PropertyDetail>(&cache_key).await? {
                    return Ok(cached);
                }
            }
            let sanitized = detail.sanitize_for_role(user.role);
            if sanitized.status == crate::domain::properties::PropertyStatus::Published {
                self.cache.set_json(&cache_key, &sanitized).await?;
            }
            return Ok(sanitized);
        }

        let cache_key = self.cache.versioned_key("properties:detail", &format!("{id}:restricted")).await?;
        if let Some(cached) = self.cache.get_json::<PropertyDetail>(&cache_key).await? {
            return Ok(cached);
        }
        let detail = self
            .properties
            .find_published_detail_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("property not found"))?
            .sanitize_for_role(UserRole::Seeker);
        self.cache.set_json(&cache_key, &detail).await?;
        Ok(detail)
    }
}
