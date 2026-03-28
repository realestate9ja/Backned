use crate::domain::properties::{CreatePropertyInput, Property, PropertyDetail, PropertyListItem};
use anyhow::Result;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Clone)]
pub struct PropertyRepository {
    pool: PgPool,
}

impl PropertyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        input: &CreatePropertyInput,
        owner_id: Uuid,
        agent_id: Option<Uuid>,
    ) -> Result<Property> {
        let property = sqlx::query_as::<_, Property>(
            r#"
            INSERT INTO properties (
                id, owner_id, agent_id, title, price, location, exact_address, description, images,
                contact_name, contact_phone, is_service_apartment, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'published')
            RETURNING id, owner_id, agent_id, title, price, location, exact_address, description, images,
                      contact_name, contact_phone, is_service_apartment, status, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(owner_id)
        .bind(agent_id)
        .bind(&input.title)
        .bind(input.price)
        .bind(&input.location)
        .bind(&input.exact_address)
        .bind(&input.description)
        .bind(&input.images)
        .bind(&input.contact_name)
        .bind(&input.contact_phone)
        .bind(input.is_service_apartment)
        .fetch_one(&self.pool)
        .await?;

        Ok(property)
    }

    pub async fn list(
        &self,
        limit: i64,
        offset: i64,
        location: Option<&str>,
        min_price: Option<i64>,
        max_price: Option<i64>,
    ) -> Result<Vec<PropertyListItem>> {
        let mut builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                p.id,
                p.title,
                p.price,
                p.location,
                p.description,
                p.images,
                p.is_service_apartment,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.created_at
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            WHERE p.status = 'published'
            "#,
        );

        if let Some(location) = location {
            builder.push(" AND p.location ILIKE ");
            builder.push_bind(format!("%{location}%"));
        }
        if let Some(min_price) = min_price {
            builder.push(" AND p.price >= ");
            builder.push_bind(min_price);
        }
        if let Some(max_price) = max_price {
            builder.push(" AND p.price <= ");
            builder.push_bind(max_price);
        }

        builder.push(" ORDER BY p.created_at DESC LIMIT ");
        builder.push_bind(limit);
        builder.push(" OFFSET ");
        builder.push_bind(offset);

        let properties = builder
            .build_query_as::<PropertyListItem>()
            .fetch_all(&self.pool)
            .await?;

        Ok(properties)
    }

    pub async fn find_detail_by_id(&self, id: Uuid) -> Result<Option<PropertyDetail>> {
        let property = sqlx::query_as::<_, PropertyDetail>(
            r#"
            SELECT
                p.id,
                p.title,
                p.price,
                p.location,
                p.description,
                p.images,
                p.is_service_apartment,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.exact_address,
                p.contact_name,
                p.contact_phone,
                p.created_at,
                p.updated_at
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            WHERE p.id = $1 AND p.status = 'published'
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(property)
    }

    pub async fn list_recent_by_owner(&self, owner_id: Uuid, limit: i64) -> Result<Vec<PropertyListItem>> {
        let items = sqlx::query_as::<_, PropertyListItem>(
            r#"
            SELECT
                p.id,
                p.title,
                p.price,
                p.location,
                p.description,
                p.images,
                p.is_service_apartment,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.created_at
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            WHERE p.owner_id = $1
            ORDER BY p.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(owner_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn list_recent_managed_by_agent(&self, agent_id: Uuid, limit: i64) -> Result<Vec<PropertyListItem>> {
        let items = sqlx::query_as::<_, PropertyListItem>(
            r#"
            SELECT
                p.id,
                p.title,
                p.price,
                p.location,
                p.description,
                p.images,
                p.is_service_apartment,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.created_at
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            WHERE p.agent_id = $1 OR p.owner_id = $1
            ORDER BY p.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn list_service_apartments_managed_by_agent(
        &self,
        agent_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PropertyListItem>> {
        let items = sqlx::query_as::<_, PropertyListItem>(
            r#"
            SELECT
                p.id,
                p.title,
                p.price,
                p.location,
                p.description,
                p.images,
                p.is_service_apartment,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.created_at
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            WHERE p.is_service_apartment = TRUE
              AND (p.agent_id = $1 OR p.owner_id = $1)
            ORDER BY p.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn list_owned_or_managed_by_user_for_ids(
        &self,
        user_id: Uuid,
        property_ids: &[Uuid],
    ) -> Result<Vec<PropertyListItem>> {
        if property_ids.is_empty() {
            return Ok(Vec::new());
        }

        let items = sqlx::query_as::<_, PropertyListItem>(
            r#"
            SELECT
                p.id,
                p.title,
                p.price,
                p.location,
                p.description,
                p.images,
                p.is_service_apartment,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.created_at
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            WHERE p.id = ANY($1)
              AND (p.owner_id = $2 OR p.agent_id = $2)
            ORDER BY p.created_at DESC
            "#,
        )
        .bind(property_ids)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }
}
