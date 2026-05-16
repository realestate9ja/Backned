use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::comments::model::{
    CommentReply, CommentWithAuthorRow, PropertyComment, ReplyWithAuthorRow,
};

pub struct CommentRepository {
    pool: PgPool,
}

impl CommentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Comments operations
    pub async fn create_comment(
        &self,
        property_id: Uuid,
        user_id: Uuid,
        content: &str,
    ) -> Result<PropertyComment> {
        let comment = sqlx::query_as::<_, PropertyComment>(
            r#"
            INSERT INTO property_comments (property_id, user_id, content)
            VALUES ($1, $2, $3)
            RETURNING id, property_id, user_id, content, created_at, updated_at, deleted_at
            "#,
        )
        .bind(property_id)
        .bind(user_id)
        .bind(content)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn get_property_comments(&self, property_id: Uuid, limit: i64, offset: i64) -> Result<Vec<PropertyComment>> {
        let comments = sqlx::query_as::<_, PropertyComment>(
            r#"
            SELECT id, property_id, user_id, content, created_at, updated_at, deleted_at
            FROM property_comments
            WHERE property_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(property_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    pub async fn get_property_comments_with_authors(
        &self,
        property_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CommentWithAuthorRow>> {
        let comments = sqlx::query_as::<_, CommentWithAuthorRow>(
            r#"
            SELECT
                c.id,
                c.property_id,
                c.user_id,
                c.content,
                c.created_at,
                c.updated_at,
                c.deleted_at,
                u.full_name AS author_name,
                u.role::text AS author_role,
                COALESCE(NULLIF(TRIM(u.wallet_address), ''), p.avatar_url) AS author_avatar
            FROM property_comments c
            INNER JOIN users u ON u.id = c.user_id
            LEFT JOIN profiles p ON p.user_id = u.id
            WHERE c.property_id = $1 AND c.deleted_at IS NULL
            ORDER BY c.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(property_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    pub async fn get_comment_count(&self, property_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM property_comments WHERE property_id = $1 AND deleted_at IS NULL"
        )
        .bind(property_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    pub async fn get_comment_by_id(&self, comment_id: Uuid) -> Result<Option<PropertyComment>> {
        let comment = sqlx::query_as::<_, PropertyComment>(
            "SELECT id, property_id, user_id, content, created_at, updated_at, deleted_at FROM property_comments WHERE id = $1"
        )
        .bind(comment_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn update_comment(&self, comment_id: Uuid, content: &str) -> Result<PropertyComment> {
        let comment = sqlx::query_as::<_, PropertyComment>(
            r#"
            UPDATE property_comments
            SET content = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, property_id, user_id, content, created_at, updated_at, deleted_at
            "#,
        )
        .bind(content)
        .bind(comment_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn delete_comment(&self, comment_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE property_comments SET deleted_at = NOW() WHERE id = $1")
            .bind(comment_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Replies operations
    pub async fn create_reply(
        &self,
        comment_id: Uuid,
        user_id: Uuid,
        content: &str,
    ) -> Result<CommentReply> {
        let reply = sqlx::query_as::<_, CommentReply>(
            r#"
            INSERT INTO comment_replies (comment_id, user_id, content)
            VALUES ($1, $2, $3)
            RETURNING id, comment_id, user_id, content, created_at, updated_at, deleted_at
            "#,
        )
        .bind(comment_id)
        .bind(user_id)
        .bind(content)
        .fetch_one(&self.pool)
        .await?;

        Ok(reply)
    }

    pub async fn get_comment_replies(&self, comment_id: Uuid) -> Result<Vec<CommentReply>> {
        let replies = sqlx::query_as::<_, CommentReply>(
            r#"
            SELECT id, comment_id, user_id, content, created_at, updated_at, deleted_at
            FROM comment_replies
            WHERE comment_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
        )
        .bind(comment_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(replies)
    }

    pub async fn get_replies_for_comment_ids_with_authors(
        &self,
        comment_ids: &[Uuid],
    ) -> Result<Vec<ReplyWithAuthorRow>> {
        if comment_ids.is_empty() {
            return Ok(Vec::new());
        }

        let replies = sqlx::query_as::<_, ReplyWithAuthorRow>(
            r#"
            SELECT
                r.id,
                r.comment_id,
                r.user_id,
                r.content,
                r.created_at,
                r.updated_at,
                r.deleted_at,
                u.full_name AS author_name,
                u.role::text AS author_role,
                COALESCE(NULLIF(TRIM(u.wallet_address), ''), p.avatar_url) AS author_avatar
            FROM comment_replies r
            INNER JOIN users u ON u.id = r.user_id
            LEFT JOIN profiles p ON p.user_id = u.id
            WHERE r.comment_id = ANY($1) AND r.deleted_at IS NULL
            ORDER BY r.comment_id ASC, r.created_at ASC
            "#,
        )
        .bind(comment_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(replies)
    }

    pub async fn get_reply_count(&self, comment_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM comment_replies WHERE comment_id = $1 AND deleted_at IS NULL"
        )
        .bind(comment_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    pub async fn get_reply_by_id(&self, reply_id: Uuid) -> Result<Option<CommentReply>> {
        let reply = sqlx::query_as::<_, CommentReply>(
            "SELECT id, comment_id, user_id, content, created_at, updated_at, deleted_at FROM comment_replies WHERE id = $1"
        )
        .bind(reply_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(reply)
    }

    pub async fn update_reply(&self, reply_id: Uuid, content: &str) -> Result<CommentReply> {
        let reply = sqlx::query_as::<_, CommentReply>(
            r#"
            UPDATE comment_replies
            SET content = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, comment_id, user_id, content, created_at, updated_at, deleted_at
            "#,
        )
        .bind(content)
        .bind(reply_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(reply)
    }

    pub async fn delete_reply(&self, reply_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE comment_replies SET deleted_at = NOW() WHERE id = $1")
            .bind(reply_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
