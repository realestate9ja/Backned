use crate::domain::audit::CreateAuditLogInput;
use anyhow::Result;
use sqlx::{types::Json, PgPool};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditLogRepository {
    pool: PgPool,
}

impl AuditLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: &CreateAuditLogInput) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                id, request_id, user_id, email, role, action, method, path, status_code, ip_address,
                user_agent, resource_type, resource_id, success, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CAST($10 AS INET), $11, $12, $13, $14, $15)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(input.request_id)
        .bind(input.user_id)
        .bind(&input.email)
        .bind(&input.role)
        .bind(&input.action)
        .bind(&input.method)
        .bind(&input.path)
        .bind(input.status_code)
        .bind(&input.ip_address)
        .bind(&input.user_agent)
        .bind(&input.resource_type)
        .bind(input.resource_id)
        .bind(input.success)
        .bind(Json(&input.metadata))
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
