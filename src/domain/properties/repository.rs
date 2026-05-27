use crate::domain::properties::{
    CreatePropertyInput, Property, PropertyDetail, PropertyListItem, PropertyStatus,
};
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
        self_managed: bool,
        status: PropertyStatus,
    ) -> Result<Property> {
        let property = sqlx::query_as::<_, Property>(
            r#"
            INSERT INTO properties (
                id, owner_id, agent_id, title, price, location, exact_address, description, images,
                contact_name, contact_phone, is_service_apartment, listing_type, self_managed, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, owner_id, agent_id, title, price, location, exact_address, description, images,
                      contact_name, contact_phone, is_service_apartment, listing_type, self_managed, status,
                      verified_by, verified_at, created_at, updated_at
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
        .bind(input.listing_type.as_deref().unwrap_or(if input.is_service_apartment { "shortlet" } else { "rent" }))
        .bind(self_managed)
        .bind(status)
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
                p.listing_type,
                p.status,
                p.self_managed,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                owner.phone AS owner_phone,
                agent.phone AS agent_phone,
                p.created_at,
                p.verified_at,
                COALESCE(view_stats.view_count, 0)::bigint AS view_count,
                COALESCE(offer_stats.offer_count, 0)::bigint AS offer_count,
                pending_change.id AS pending_price_request_id,
                pending_change.requested_price AS pending_requested_price,
                pending_change.created_at AS pending_price_requested_at,
                status_lock.available_at AS status_locked_until
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            LEFT JOIN (
                SELECT property_id, COUNT(*)::bigint AS view_count
                FROM property_views
                GROUP BY property_id
            ) view_stats ON view_stats.property_id = p.id
            LEFT JOIN (
                SELECT property_id, COUNT(*)::bigint AS offer_count
                FROM offers
                GROUP BY property_id
            ) offer_stats ON offer_stats.property_id = p.id
            LEFT JOIN property_change_requests pending_change
                ON pending_change.property_id = p.id
               AND pending_change.request_type = 'price_increase'
               AND pending_change.status = 'pending'
            LEFT JOIN LATERAL (
                SELECT prp.available_at
                FROM property_rental_periods prp
                WHERE prp.property_id = p.id AND prp.available_at IS NOT NULL
                ORDER BY prp.available_at DESC
                LIMIT 1
            ) status_lock ON TRUE
            WHERE p.status IN ('published', 'rented_out', 'sold_out', 'in_use')
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

    pub async fn find_published_detail_by_id(&self, id: Uuid) -> Result<Option<PropertyDetail>> {
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
                p.listing_type,
                p.status,
                p.self_managed,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                agent.phone AS agent_phone,
                ap.company_name,
                p.exact_address,
                p.contact_name,
                p.contact_phone,
                p.verified_by,
                p.verified_at,
                p.created_at,
                p.updated_at,
                COALESCE(view_stats.view_count, 0) AS view_count,
                COALESCE(offer_stats.offer_count, 0) AS offer_count,
                pending_change.id AS pending_price_request_id,
                pending_change.requested_price AS pending_requested_price,
                pending_change.created_at AS pending_price_requested_at,
                status_lock.available_at AS status_locked_until
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            LEFT JOIN agent_profiles ap ON ap.user_id = p.agent_id
            LEFT JOIN (
                SELECT property_id, COUNT(*)::bigint AS view_count
                FROM property_views
                GROUP BY property_id
            ) view_stats ON view_stats.property_id = p.id
            LEFT JOIN (
                SELECT property_id, COUNT(*)::bigint AS offer_count
                FROM offers
                GROUP BY property_id
            ) offer_stats ON offer_stats.property_id = p.id
            LEFT JOIN property_change_requests pending_change
                ON pending_change.property_id = p.id
               AND pending_change.request_type = 'price_increase'
               AND pending_change.status = 'pending'
            LEFT JOIN LATERAL (
                SELECT prp.available_at
                FROM property_rental_periods prp
                WHERE prp.property_id = p.id AND prp.available_at IS NOT NULL
                ORDER BY prp.available_at DESC
                LIMIT 1
            ) status_lock ON TRUE
            WHERE p.id = $1 AND p.status IN ('published', 'rented_out', 'sold_out', 'in_use')
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(property)
    }

    pub async fn find_detail_by_id_including_unpublished(
        &self,
        id: Uuid,
    ) -> Result<Option<PropertyDetail>> {
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
                p.listing_type,
                p.status,
                p.self_managed,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                agent.phone AS agent_phone,
                ap.company_name,
                p.exact_address,
                p.contact_name,
                p.contact_phone,
                p.verified_by,
                p.verified_at,
                p.created_at,
                p.updated_at,
                COALESCE(view_stats.view_count, 0) AS view_count,
                COALESCE(offer_stats.offer_count, 0) AS offer_count,
                pending_change.id AS pending_price_request_id,
                pending_change.requested_price AS pending_requested_price,
                pending_change.created_at AS pending_price_requested_at,
                status_lock.available_at AS status_locked_until
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            LEFT JOIN agent_profiles ap ON ap.user_id = p.agent_id
            LEFT JOIN (
                SELECT property_id, COUNT(*)::bigint AS view_count
                FROM property_views
                GROUP BY property_id
            ) view_stats ON view_stats.property_id = p.id
            LEFT JOIN (
                SELECT property_id, COUNT(*)::bigint AS offer_count
                FROM offers
                GROUP BY property_id
            ) offer_stats ON offer_stats.property_id = p.id
            LEFT JOIN property_change_requests pending_change
                ON pending_change.property_id = p.id
               AND pending_change.request_type = 'price_increase'
               AND pending_change.status = 'pending'
            LEFT JOIN LATERAL (
                SELECT prp.available_at
                FROM property_rental_periods prp
                WHERE prp.property_id = p.id AND prp.available_at IS NOT NULL
                ORDER BY prp.available_at DESC
                LIMIT 1
            ) status_lock ON TRUE
            WHERE p.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(property)
    }

    pub async fn list_recent_by_owner(
        &self,
        owner_id: Uuid,
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
                p.status,
                p.self_managed,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.created_at,
                p.verified_at
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

    pub async fn list_recent_by_owner_and_status(
        &self,
        owner_id: Uuid,
        status: PropertyStatus,
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
                p.status,
                p.self_managed,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.created_at,
                p.verified_at
            FROM properties p
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            WHERE p.owner_id = $1 AND p.status = $2
            ORDER BY p.created_at DESC
            LIMIT $3
            "#,
        )
        .bind(owner_id)
        .bind(status)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn list_recent_managed_by_agent(
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
                p.status,
                p.self_managed,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.created_at,
                p.verified_at
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
                p.status,
                p.self_managed,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.created_at,
                p.verified_at
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
                p.status,
                p.self_managed,
                p.owner_id,
                p.agent_id,
                owner.full_name AS owner_name,
                agent.full_name AS agent_name,
                p.created_at,
                p.verified_at
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

    pub async fn assign_agent(
        &self,
        property_id: Uuid,
        agent_id: Uuid,
    ) -> Result<Option<Property>> {
        let property = sqlx::query_as::<_, Property>(
            r#"
            UPDATE properties
            SET agent_id = $2,
                self_managed = FALSE,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, owner_id, agent_id, title, price, location, exact_address, description, images,
                      contact_name, contact_phone, is_service_apartment, self_managed, status,
                      verified_by, verified_at, created_at, updated_at
            "#,
        )
        .bind(property_id)
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(property)
    }

    pub async fn verify_property(
        &self,
        property_id: Uuid,
        verifier_id: Uuid,
    ) -> Result<Option<Property>> {
        let property = sqlx::query_as::<_, Property>(
            r#"
            UPDATE properties
            SET status = 'verified',
                verified_by = $2,
                verified_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, owner_id, agent_id, title, price, location, exact_address, description, images,
                      contact_name, contact_phone, is_service_apartment, self_managed, status,
                      verified_by, verified_at, created_at, updated_at
            "#,
        )
        .bind(property_id)
        .bind(verifier_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(property)
    }

    pub async fn publish_property(&self, property_id: Uuid) -> Result<Option<Property>> {
        let property = sqlx::query_as::<_, Property>(
            r#"
            UPDATE properties
            SET status = 'published',
                updated_at = NOW()
            WHERE id = $1 AND status = 'verified'
            RETURNING id, owner_id, agent_id, title, price, location, exact_address, description, images,
                      contact_name, contact_phone, is_service_apartment, self_managed, status,
                      verified_by, verified_at, created_at, updated_at
            "#,
        )
        .bind(property_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(property)
    }

    pub async fn suspend_property(&self, property_id: Uuid) -> Result<Option<Property>> {
        let property = sqlx::query_as::<_, Property>(
            r#"
            UPDATE properties
            SET status = 'suspended',
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, owner_id, agent_id, title, price, location, exact_address, description, images,
                      contact_name, contact_phone, is_service_apartment, self_managed, status,
                      verified_by, verified_at, created_at, updated_at
            "#,
        )
        .bind(property_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(property)
    }

    pub async fn hide_property(&self, property_id: Uuid) -> Result<Option<Property>> {
        let property = sqlx::query_as::<_, Property>(
            r#"
            UPDATE properties
            SET status = 'hidden',
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, owner_id, agent_id, title, price, location, exact_address, description, images,
                      contact_name, contact_phone, is_service_apartment, self_managed, status,
                      verified_by, verified_at, created_at, updated_at
            "#,
        )
        .bind(property_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(property)
    }

    pub async fn record_view(&self, property_id: Uuid, viewer_user_id: Option<Uuid>) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO property_views (id, property_id, viewer_user_id)
            SELECT $1, $2, $3
            WHERE
                -- Allow anonymous views (null viewer)
                $3 IS NULL
                OR (
                    -- For authenticated users, only allow if:
                    -- 1. They are NOT the owner or agent of this property
                    NOT EXISTS (
                        SELECT 1
                        FROM properties p
                        WHERE p.id = $2
                          AND (p.owner_id = $3 OR p.agent_id = $3)
                    )
                    -- 2. AND they haven't viewed this property before
                    AND NOT EXISTS (
                        SELECT 1
                        FROM property_views pv
                        WHERE pv.property_id = $2
                          AND pv.viewer_user_id = $3
                    )
                )
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(property_id)
        .bind(viewer_user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
