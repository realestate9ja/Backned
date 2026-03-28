use crate::domain::notifications::{AgentNotificationTarget, AgentPostNotificationItem};
use anyhow::Result;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationRepository {
    pool: PgPool,
}

impl NotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_for_post(&self, post_id: Uuid, targets: &[AgentNotificationTarget]) -> Result<()> {
        if targets.is_empty() {
            return Ok(());
        }

        let mut builder = QueryBuilder::<Postgres>::new(
            "INSERT INTO agent_post_notifications (id, agent_id, post_id, matched_city, matched_state) ",
        );
        builder.push_values(targets, |mut row, target| {
            row.push_bind(Uuid::new_v4())
                .push_bind(target.agent_id)
                .push_bind(post_id)
                .push_bind(&target.matched_city)
                .push_bind(&target.matched_state);
        });
        builder.push(" ON CONFLICT (agent_id, post_id) DO NOTHING");
        builder.build().execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_for_agent(&self, agent_id: Uuid, limit: i64) -> Result<Vec<AgentPostNotificationItem>> {
        let items = sqlx::query_as::<_, AgentPostNotificationItem>(
            r#"
            SELECT
                n.id AS notification_id,
                p.id AS post_id,
                p.author_id,
                u.full_name AS author_name,
                u.role::text AS author_role,
                p.budget,
                p.location,
                p.city,
                p.state,
                p.description,
                COALESCE(n.matched_city, '') AS matched_city,
                COALESCE(n.matched_state, '') AS matched_state,
                n.is_read,
                n.created_at
            FROM agent_post_notifications n
            INNER JOIN posts p ON p.id = n.post_id
            INNER JOIN users u ON u.id = p.author_id
            WHERE n.agent_id = $1
            ORDER BY n.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn count_unread_for_agent(&self, agent_id: Uuid) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)::bigint FROM agent_post_notifications WHERE agent_id = $1 AND is_read = FALSE",
        )
        .bind(agent_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}
