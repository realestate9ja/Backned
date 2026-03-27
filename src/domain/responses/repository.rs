use crate::domain::responses::{CreateResponseInput, Response};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ResponseRepository {
    pool: PgPool,
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
    ) -> Result<Response> {
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

        Ok(response)
    }
}

