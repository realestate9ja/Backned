use crate::domain::{
    properties::PropertyListItem,
    responses::{
        CreateResponseInput, PostResponseItem, PostResponseWithProperties, Response, ResponseContext,
        ResponseCreated,
    },
};
use anyhow::Result;
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct ResponseRepository {
    pool: PgPool,
}

#[derive(sqlx::FromRow)]
struct ResponsePropertyRow {
    response_id: Uuid,
    id: Uuid,
    title: String,
    price: i64,
    location: String,
    description: String,
    images: Vec<String>,
    is_service_apartment: bool,
    status: crate::domain::properties::PropertyStatus,
    self_managed: bool,
    owner_id: Uuid,
    agent_id: Option<Uuid>,
    owner_name: String,
    agent_name: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    verified_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ResponseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        post_id: Uuid,
        responder_id: Uuid,
        input: &CreateResponseInput,
    ) -> Result<ResponseCreated> {
        let response = sqlx::query_as::<_, Response>(
            r#"
            INSERT INTO responses (id, post_id, responder_id, message)
            VALUES ($1, $2, $3, $4)
            RETURNING id, post_id, responder_id, message, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(post_id)
        .bind(responder_id)
        .bind(&input.message)
        .fetch_one(&self.pool)
        .await?;

        if !input.property_ids.is_empty() {
            let mut builder = QueryBuilder::<Postgres>::new(
                "INSERT INTO response_properties (response_id, property_id) ",
            );
            builder.push_values(input.property_ids.iter(), |mut row, property_id| {
                row.push_bind(response.id).push_bind(*property_id);
            });
            builder.build().execute(&self.pool).await?;
        }

        let properties = self.properties_for_response_ids(&[response.id]).await?;
        Ok(ResponseCreated {
            id: response.id,
            post_id: response.post_id,
            responder_id: response.responder_id,
            message: response.message,
            properties: properties.get(&response.id).cloned().unwrap_or_default(),
            created_at: response.created_at,
        })
    }

    pub async fn list_with_properties_for_post(&self, post_id: Uuid) -> Result<Vec<PostResponseWithProperties>> {
        let items = sqlx::query_as::<_, PostResponseItem>(
            r#"
            SELECT
                r.id AS response_id,
                r.responder_id,
                u.full_name AS responder_name,
                u.role::text AS responder_role,
                r.message,
                r.created_at
            FROM responses r
            INNER JOIN users u ON u.id = r.responder_id
            WHERE r.post_id = $1
            ORDER BY r.created_at DESC
            "#,
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await?;

        let response_ids = items.iter().map(|item| item.response_id).collect::<Vec<_>>();
        let properties = self.properties_for_response_ids(&response_ids).await?;

        Ok(items
            .into_iter()
            .map(|item| PostResponseWithProperties {
                response_id: item.response_id,
                post_id,
                responder_id: item.responder_id,
                responder_name: item.responder_name,
                responder_role: item.responder_role,
                message: item.message,
                properties: properties.get(&item.response_id).cloned().unwrap_or_default(),
                created_at: item.created_at,
            })
            .collect())
    }

    pub async fn find_context(&self, response_id: Uuid) -> Result<Option<ResponseContext>> {
        let item = sqlx::query_as::<_, ResponseContext>(
            r#"
            SELECT
                p.author_id AS post_author_id,
                r.responder_id
            FROM responses r
            INNER JOIN posts p ON p.id = r.post_id
            WHERE r.id = $1
            "#,
        )
        .bind(response_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(item)
    }

    async fn properties_for_response_ids(
        &self,
        response_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<PropertyListItem>>> {
        if response_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query_as::<_, ResponsePropertyRow>(
            r#"
            SELECT
                rp.response_id,
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
            FROM response_properties rp
            INNER JOIN properties p ON p.id = rp.property_id
            INNER JOIN users owner ON owner.id = p.owner_id
            LEFT JOIN users agent ON agent.id = p.agent_id
            WHERE rp.response_id = ANY($1)
            ORDER BY p.created_at DESC
            "#,
        )
        .bind(response_ids)
        .fetch_all(&self.pool)
        .await?;

        let mut grouped: HashMap<Uuid, Vec<PropertyListItem>> = HashMap::new();
        for row in rows {
            grouped.entry(row.response_id).or_default().push(PropertyListItem {
                id: row.id,
                title: row.title,
                price: row.price,
                location: row.location,
                description: row.description,
                images: row.images,
                is_service_apartment: row.is_service_apartment,
                status: row.status,
                self_managed: row.self_managed,
                owner_id: row.owner_id,
                agent_id: row.agent_id,
                owner_name: row.owner_name,
                agent_name: row.agent_name,
                created_at: row.created_at,
                verified_at: row.verified_at,
            });
        }

        Ok(grouped)
    }
}
