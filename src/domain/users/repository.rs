use crate::domain::users::{
    AgentNotificationRecipient, AgentProfile, RegisterUserInput, UpdateAgentNotificationSettingsInput, User,
};
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
            RETURNING id, full_name, email, password_hash, role, phone, bio,
                      notifications_enabled, operating_city, operating_state, created_at, updated_at
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
            SELECT id, full_name, email, password_hash, role, phone, bio,
                   notifications_enabled, operating_city, operating_state, created_at, updated_at
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
            SELECT id, full_name, email, password_hash, role, phone, bio,
                   notifications_enabled, operating_city, operating_state, created_at, updated_at
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
            SELECT id, full_name, email, bio, operating_city, operating_state, created_at
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
            SELECT id, full_name, email, password_hash, role, phone, bio,
                   notifications_enabled, operating_city, operating_state, created_at, updated_at
            FROM users
            WHERE id = $1 AND role = 'agent'
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_agent_notification_settings(
        &self,
        agent_id: Uuid,
        input: &UpdateAgentNotificationSettingsInput,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET notifications_enabled = $2,
                operating_city = $3,
                operating_state = $4,
                updated_at = NOW()
            WHERE id = $1 AND role = 'agent'
            RETURNING id, full_name, email, password_hash, role, phone, bio,
                      notifications_enabled, operating_city, operating_state, created_at, updated_at
            "#,
        )
        .bind(agent_id)
        .bind(input.notifications_enabled)
        .bind(input.operating_city.as_ref().map(|value| value.trim().to_string()))
        .bind(input.operating_state.as_ref().map(|value| value.trim().to_string()))
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn list_notifiable_agents(
        &self,
        city: &str,
        state: &str,
    ) -> Result<Vec<AgentNotificationRecipient>> {
        let recipients = sqlx::query_as::<_, AgentNotificationRecipient>(
            r#"
            SELECT
                id,
                COALESCE(operating_city, '') AS operating_city,
                COALESCE(operating_state, '') AS operating_state
            FROM users
            WHERE role = 'agent'
              AND notifications_enabled = TRUE
              AND (
                    LOWER(COALESCE(operating_city, '')) = LOWER($1)
                 OR LOWER(COALESCE(operating_state, '')) = LOWER($2)
              )
            "#,
        )
        .bind(city.trim())
        .bind(state.trim())
        .fetch_all(&self.pool)
        .await?;

        Ok(recipients)
    }
}
