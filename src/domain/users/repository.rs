use crate::domain::users::{AgentProfile, RegisterUserInput, User};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: &RegisterUserInput, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, full_name, email, password_hash, role, phone, bio)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, full_name, email, password_hash, role, phone, bio, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&input.full_name)
        .bind(input.email.to_lowercase())
        .bind(password_hash)
        .bind(input.role)
        .bind(&input.phone)
        .bind(&input.bio)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, full_name, email, password_hash, role, phone, bio, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email.to_lowercase())
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, full_name, email, password_hash, role, phone, bio, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn list_agents(&self, limit: i64, offset: i64) -> Result<Vec<AgentProfile>> {
        let agents = sqlx::query_as::<_, AgentProfile>(
            r#"
            SELECT id, full_name, email, bio, created_at
            FROM users
            WHERE role = 'agent'
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(agents)
    }

    pub async fn find_agent_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, full_name, email, password_hash, role, phone, bio, created_at, updated_at
            FROM users
            WHERE id = $1 AND role = 'agent'
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }
}
