use crate::domain::users::{
    AgentNotificationRecipient, AgentProfile, BootstrapAdminInput, RegisterUserInput,
    UpdateAgentNotificationSettingsInput, User, UserRole,
};
use anyhow::Result;
use chrono::{Duration, Utc};
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
        let verification_status = if matches!(input.role, UserRole::Agent | UserRole::Landlord) {
            "pending"
        } else {
            "not_required"
        };
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, full_name, email, email_verified, password_hash, role, phone, bio, verification_status)
            VALUES ($1, $2, $3, FALSE, $4, $5, $6, $7, $8)
            RETURNING id, full_name, email, email_verified, password_hash, role, phone, bio,
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

        self.ensure_profile(
            user.id,
            &user.full_name,
            user.phone.as_deref(),
            user.bio.as_deref(),
        )
        .await?;

        Ok(user)
    }

    pub async fn create_admin(&self, input: &BootstrapAdminInput, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, full_name, email, email_verified, password_hash, role, verification_status, verified_at)
            VALUES ($1, $2, $3, TRUE, $4, 'admin', 'verified', NOW())
            RETURNING id, full_name, email, email_verified, password_hash, role, phone, bio,
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

        self.ensure_profile(user.id, &user.full_name, user.phone.as_deref(), user.bio.as_deref())
            .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, full_name, email, email_verified, password_hash, role, phone, bio,
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
            SELECT id, full_name, email, email_verified, password_hash, role, phone, bio,
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

    pub async fn update_role(&self, user_id: Uuid, role: UserRole) -> Result<Option<User>> {
        let verification_status = if matches!(role, UserRole::Agent | UserRole::Landlord) {
            "pending"
        } else {
            "not_required"
        };
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET role = $2,
                verification_status = $3,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, full_name, email, email_verified, password_hash, role, phone, bio,
                      notifications_enabled, operating_city, operating_state,
                      verification_status, verification_notes, verified_at,
                      quality_strikes, fraud_strikes, listing_restricted_until, is_banned,
                      created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(role)
        .bind(verification_status)
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
            SELECT id, full_name, email, email_verified, password_hash, role, phone, bio,
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
            RETURNING id, full_name, email, email_verified, password_hash, role, phone, bio,
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
            RETURNING id, full_name, email, email_verified, password_hash, role, phone, bio,
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
            RETURNING id, full_name, email, email_verified, password_hash, role, phone, bio,
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
            RETURNING id, full_name, email, email_verified, password_hash, role, phone, bio,
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

    pub async fn create_email_verification_token(&self, user_id: Uuid) -> Result<String> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::hours(24);
        sqlx::query(
            r#"
            INSERT INTO email_verification_tokens (id, user_id, token, expires_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(token)
    }

    pub async fn create_email_verification_code(
        &self,
        user_id: Uuid,
        email: &str,
        purpose: &str,
    ) -> Result<String> {
        let value = format!("{:05}", rand::random::<u32>() % 100_000);
        let expires_at = Utc::now() + Duration::minutes(10);
        sqlx::query(
            r#"
            INSERT INTO email_verification_codes (id, user_id, email, purpose, code, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(email.to_lowercase())
        .bind(purpose.trim())
        .bind(&value)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(value)
    }

    pub async fn find_by_email_verification_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT u.id, u.full_name, u.email, u.email_verified, u.password_hash, u.role, u.phone, u.bio,
                   u.notifications_enabled, u.operating_city, u.operating_state,
                   u.verification_status, u.verification_notes, u.verified_at,
                   u.quality_strikes, u.fraud_strikes, u.listing_restricted_until, u.is_banned,
                   u.created_at, u.updated_at
            FROM email_verification_codes evc
            INNER JOIN users u ON u.id = evc.user_id
            WHERE evc.email = $1
              AND evc.code = $2
              AND evc.used_at IS NULL
              AND evc.expires_at > NOW()
            ORDER BY evc.created_at DESC
            LIMIT 1
            "#,
        )
        .bind(email.to_lowercase())
        .bind(code.trim())
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn mark_email_verification_code_used(&self, email: &str, code: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE email_verification_codes
            SET used_at = NOW()
            WHERE email = $1 AND code = $2 AND used_at IS NULL
            "#,
        )
        .bind(email.to_lowercase())
        .bind(code.trim())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_email_verification_token(&self, token: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT u.id, u.full_name, u.email, u.email_verified, u.password_hash, u.role, u.phone, u.bio,
                   u.notifications_enabled, u.operating_city, u.operating_state,
                   u.verification_status, u.verification_notes, u.verified_at,
                   u.quality_strikes, u.fraud_strikes, u.listing_restricted_until, u.is_banned,
                   u.created_at, u.updated_at
            FROM email_verification_tokens evt
            JOIN users u ON u.id = evt.user_id
            WHERE evt.token = $1
              AND evt.used_at IS NULL
              AND evt.expires_at > NOW()
            ORDER BY evt.created_at DESC
            LIMIT 1
            "#,
        )
        .bind(token.trim())
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn mark_email_verified(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET email_verified = TRUE,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, full_name, email, email_verified, password_hash, role, phone, bio,
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

    pub async fn mark_email_verification_token_used(&self, token: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE email_verification_tokens
            SET used_at = NOW()
            WHERE token = $1 AND used_at IS NULL
            "#,
        )
        .bind(token.trim())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_refresh_token(&self, user_id: Uuid, expires_at: chrono::DateTime<Utc>) -> Result<String> {
        let token = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token, expires_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(token)
    }

    pub async fn consume_refresh_token(&self, token: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            WITH consumed AS (
                UPDATE refresh_tokens
                SET revoked_at = NOW()
                WHERE token = $1
                  AND revoked_at IS NULL
                  AND expires_at > NOW()
                RETURNING user_id
            )
            SELECT u.id, u.full_name, u.email, u.email_verified, u.password_hash, u.role, u.phone, u.bio,
                   u.notifications_enabled, u.operating_city, u.operating_state,
                   u.verification_status, u.verification_notes, u.verified_at,
                   u.quality_strikes, u.fraud_strikes, u.listing_restricted_until, u.is_banned,
                   u.created_at, u.updated_at
            FROM consumed
            INNER JOIN users u ON u.id = consumed.user_id
            "#,
        )
        .bind(token.trim())
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn revoke_refresh_token(&self, token: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE token = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(token.trim())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn ensure_profile(
        &self,
        user_id: Uuid,
        full_name: &str,
        phone: Option<&str>,
        bio: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO profiles (id, user_id, full_name, phone, bio, onboarding_completed)
            VALUES ($1, $1, $2, $3, $4, FALSE)
            ON CONFLICT (user_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(full_name)
        .bind(phone)
        .bind(bio)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
