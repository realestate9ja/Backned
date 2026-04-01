use crate::domain::users::{
    AgentNotificationRecipient, AgentProfile, BootstrapAdminInput, RegisterUserInput,
    UpdateAgentNotificationSettingsInput, User, UserRole,
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
        let verification_status = if input.role == UserRole::Agent {
            "pending"
        } else {
            "not_required"
        };
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, full_name, email, password_hash, role, phone, bio, verification_status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, full_name, email, password_hash, role, phone, bio,
                      notifications_enabled, operating_city, operating_state,
                      verification_status, verification_notes, verified_at,
                      quality_strikes, fraud_strikes, listing_restricted_until, is_banned,
                      created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&input.full_name)
        .bind(input.email.to_lowercase())
        .bind(password_hash)
        .bind(input.role)
        .bind(&input.phone)
        .bind(&input.bio)
        .bind(verification_status)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create_admin(&self, input: &BootstrapAdminInput, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, full_name, email, password_hash, role, verification_status, verified_at)
            VALUES ($1, $2, $3, $4, 'admin', 'verified', NOW())
            RETURNING id, full_name, email, password_hash, role, phone, bio,
                      notifications_enabled, operating_city, operating_state,
                      verification_status, verification_notes, verified_at,
                      quality_strikes, fraud_strikes, listing_restricted_until, is_banned,
                      created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&input.full_name)
        .bind(input.email.to_lowercase())
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, full_name, email, password_hash, role, phone, bio,
                   notifications_enabled, operating_city, operating_state,
                   verification_status, verification_notes, verified_at,
                   quality_strikes, fraud_strikes, listing_restricted_until, is_banned,
                   created_at, updated_at
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
                   notifications_enabled, operating_city, operating_state,
                   verification_status, verification_notes, verified_at,
                   quality_strikes, fraud_strikes, listing_restricted_until, is_banned,
                   created_at, updated_at
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
            SELECT
                u.id,
                u.full_name,
                u.email,
                u.bio,
                u.operating_city,
                u.operating_state,
                AVG(r.rating)::double precision AS average_rating,
                COUNT(r.id)::bigint AS review_count,
                u.verification_status,
                u.created_at
            FROM users u
            LEFT JOIN reviews r ON r.reviewee_id = u.id
            WHERE u.role = 'agent'
            GROUP BY u.id
            ORDER BY u.created_at DESC
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
                   notifications_enabled, operating_city, operating_state,
                   verification_status, verification_notes, verified_at,
                   quality_strikes, fraud_strikes, listing_restricted_until, is_banned,
                   created_at, updated_at
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
                      notifications_enabled, operating_city, operating_state,
                      verification_status, verification_notes, verified_at,
                      quality_strikes, fraud_strikes, listing_restricted_until, is_banned,
                      created_at, updated_at
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

    pub async fn apply_quality_violation(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET quality_strikes = quality_strikes + 1,
                listing_restricted_until = CASE
                    WHEN quality_strikes + 1 >= 3 THEN NOW() + INTERVAL '30 days'
                    WHEN quality_strikes + 1 >= 2 THEN NOW() + INTERVAL '7 days'
                    ELSE listing_restricted_until
                END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, full_name, email, password_hash, role, phone, bio,
                      notifications_enabled, operating_city, operating_state,
                      verification_status, verification_notes, verified_at,
                      quality_strikes, fraud_strikes, listing_restricted_until, is_banned,
                      created_at, updated_at
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn apply_fraud_violation(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET fraud_strikes = fraud_strikes + 1,
                listing_restricted_until = NOW() + INTERVAL '30 days',
                is_banned = CASE WHEN fraud_strikes + 1 >= 3 THEN TRUE ELSE is_banned END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, full_name, email, password_hash, role, phone, bio,
                      notifications_enabled, operating_city, operating_state,
                      verification_status, verification_notes, verified_at,
                      quality_strikes, fraud_strikes, listing_restricted_until, is_banned,
                      created_at, updated_at
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_agent_verification(
        &self,
        agent_id: Uuid,
        verification_status: &str,
        verification_notes: Option<&str>,
    ) -> Result<Option<User>> {
        let verified_at = (verification_status == "verified").then_some(chrono::Utc::now());
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET verification_status = $2,
                verification_notes = $3,
                verified_at = $4,
                updated_at = NOW()
            WHERE id = $1 AND role = 'agent'
            RETURNING id, full_name, email, password_hash, role, phone, bio,
                      notifications_enabled, operating_city, operating_state,
                      verification_status, verification_notes, verified_at,
                      quality_strikes, fraud_strikes, listing_restricted_until, is_banned,
                      created_at, updated_at
            "#,
        )
        .bind(agent_id)
        .bind(verification_status)
        .bind(verification_notes)
        .bind(verified_at)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }
}
