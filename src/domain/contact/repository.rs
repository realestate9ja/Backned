use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::contact::ContactMessage;

pub struct ContactMessageRepository {
    pool: PgPool,
}

impl ContactMessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_message(
        &self,
        name: &str,
        email: &str,
        subject: &str,
        message: &str,
    ) -> Result<ContactMessage> {
        let contact_msg = sqlx::query_as::<_, ContactMessage>(
            r#"
            INSERT INTO contact_messages (name, email, subject, message)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, email, subject, message, created_at, read_at, is_read
            "#,
        )
        .bind(name)
        .bind(email)
        .bind(subject)
        .bind(message)
        .fetch_one(&self.pool)
        .await?;

        Ok(contact_msg)
    }

    pub async fn get_all_messages(&self, limit: i64, offset: i64) -> Result<Vec<ContactMessage>> {
        let messages = sqlx::query_as::<_, ContactMessage>(
            r#"
            SELECT id, name, email, subject, message, created_at, read_at, is_read
            FROM contact_messages
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }

    pub async fn get_unread_count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM contact_messages WHERE is_read = FALSE"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    pub async fn mark_as_read(&self, message_id: Uuid) -> Result<ContactMessage> {
        let message = sqlx::query_as::<_, ContactMessage>(
            r#"
            UPDATE contact_messages
            SET is_read = TRUE, read_at = NOW()
            WHERE id = $1
            RETURNING id, name, email, subject, message, created_at, read_at, is_read
            "#,
        )
        .bind(message_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(message)
    }

    pub async fn get_message_by_id(&self, message_id: Uuid) -> Result<Option<ContactMessage>> {
        let message = sqlx::query_as::<_, ContactMessage>(
            "SELECT id, name, email, subject, message, created_at, read_at, is_read FROM contact_messages WHERE id = $1"
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(message)
    }

    pub async fn get_total_count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM contact_messages"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }
}
