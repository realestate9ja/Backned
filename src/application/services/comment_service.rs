use std::sync::Arc;

use anyhow::Result;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{
        comments::{CommentRepository, CreateCommentInput, CreateReplyInput, CommentWithAuthor, CommentWithReplies, ReplyWithAuthor},
        users::UserRepository,
    },
};

pub struct CommentService {
    repo: Arc<CommentRepository>,
    users: UserRepository,
    pool: PgPool,
}

impl CommentService {
    pub fn new(
        repo: Arc<CommentRepository>,
        users: UserRepository,
        pool: PgPool,
    ) -> Self {
        Self {
            repo,
            users,
            pool,
        }
    }

    async fn user_avatar_url(&self, user_id: Uuid) -> Result<Option<String>> {
        self.users.find_avatar_url(user_id).await.map_err(Into::into)
    }

    pub async fn create_comment(
        &self,
        property_id: Uuid,
        user_id: Uuid,
        input: CreateCommentInput,
    ) -> Result<CommentWithAuthor> {
        // Create the comment
        let comment = self.repo.create_comment(property_id, user_id, &input.content).await?;

        // Get comment author info
        let author = self.users.find_by_id(user_id).await?
            .ok_or(anyhow::anyhow!("User not found"))?;
        let author_role = author.role.as_str().to_string();
        let avatar = self.user_avatar_url(user_id).await?;

        // Get property agent to notify
        let property_agent: (Uuid,) = sqlx::query_as(
            "SELECT agent_id FROM properties WHERE id = $1"
        )
        .bind(property_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(anyhow::anyhow!("Property not found"))?;

        // Send notification to property agent (if different from commenter)
        if property_agent.0 != user_id {
            let notification_title = "New comment on your property";
            let truncated_content = if input.content.len() > 100 {
                format!("{}...", &input.content[..100])
            } else {
                input.content.clone()
            };
            let notification_body = format!("{} commented: \"{}\"", author.full_name, truncated_content);

            let _ = sqlx::query(
                "INSERT INTO notifications (id, user_id, type, title, body, data_json) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(Uuid::new_v4())
            .bind(property_agent.0)
            .bind("comment")
            .bind(&notification_title)
            .bind(&notification_body)
            .bind(json!({
                "property_id": property_id,
                "comment_id": comment.id,
                "commenter_name": author.full_name,
                "actionUrl": format!("/properties/{}#comment-{}", property_id, comment.id)
            }))
            .execute(&self.pool)
            .await;
        }

        Ok(CommentWithAuthor {
            comment,
            author_name: author.full_name,
            author_role,
            author_avatar: avatar,
        })
    }

    pub async fn create_reply(
        &self,
        comment_id: Uuid,
        user_id: Uuid,
        input: CreateReplyInput,
    ) -> Result<ReplyWithAuthor> {
        // Get the original comment
        let comment = self.repo.get_comment_by_id(comment_id).await?
            .ok_or(anyhow::anyhow!("Comment not found"))?;

        // Create reply
        let reply = self.repo.create_reply(comment_id, user_id, &input.content).await?;

        // Get reply author info
        let author = self.users.find_by_id(user_id).await?
            .ok_or(anyhow::anyhow!("User not found"))?;
        let author_role = author.role.as_str().to_string();
        let avatar = self.user_avatar_url(user_id).await?;

        let truncated_content = if input.content.len() > 100 {
            format!("{}...", &input.content[..100])
        } else {
            input.content.clone()
        };

        // Notify original comment author (if different from replier)
        if comment.user_id != user_id {
            let notification_title = "Someone replied to your comment";
            let notification_body = format!("{} replied: \"{}\"", author.full_name, truncated_content);

            let _ = sqlx::query(
                "INSERT INTO notifications (id, user_id, type, title, body, data_json) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(Uuid::new_v4())
            .bind(comment.user_id)
            .bind("comment_reply")
            .bind(notification_title)
            .bind(&notification_body)
            .bind(json!({
                "property_id": comment.property_id,
                "comment_id": comment_id,
                "reply_id": reply.id,
                "replier_name": author.full_name,
                "actionUrl": format!("/properties/{}#comment-{}", comment.property_id, comment_id)
            }))
            .execute(&self.pool)
            .await;
        }

        // Also notify property agent (if they're not the replier and not the original commenter)
        let property_agent: (Uuid,) = sqlx::query_as(
            "SELECT agent_id FROM properties WHERE id = $1"
        )
        .bind(comment.property_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(anyhow::anyhow!("Property not found"))?;

        if property_agent.0 != user_id && property_agent.0 != comment.user_id {
            let notification_title = "New reply on your property comment thread";
            let notification_body = format!("{} replied: \"{}\"", author.full_name, truncated_content);

            let _ = sqlx::query(
                "INSERT INTO notifications (id, user_id, type, title, body, data_json) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(Uuid::new_v4())
            .bind(property_agent.0)
            .bind("comment_reply")
            .bind(&notification_title)
            .bind(&notification_body)
            .bind(json!({
                "property_id": comment.property_id,
                "comment_id": comment_id,
                "reply_id": reply.id,
                "replier_name": author.full_name,
                "actionUrl": format!("/properties/{}#comment-{}", comment.property_id, comment_id)
            }))
            .execute(&self.pool)
            .await;
        }

        Ok(ReplyWithAuthor {
            reply,
            author_name: author.full_name,
            author_role,
            author_avatar: avatar,
        })
    }

    pub async fn get_comments_with_replies(
        &self,
        property_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CommentWithReplies>> {
        let comments = self.repo.get_property_comments(property_id, limit, offset).await?;
        let mut result = Vec::new();

        for comment in comments {
            // Get comment author info
            let author = self.users.find_by_id(comment.user_id).await?
                .ok_or(anyhow::anyhow!("User not found"))?;
            let author_role = author.role.as_str().to_string();
            let avatar = self.user_avatar_url(comment.user_id).await?;

            let comment_with_author = CommentWithAuthor {
                comment: comment.clone(),
                author_name: author.full_name,
                author_role,
                author_avatar: avatar,
            };

            // Get replies for this comment
            let replies = self.repo.get_comment_replies(comment.id).await?;
            let mut replies_with_authors = Vec::new();

            for reply in replies {
                let reply_author = self.users.find_by_id(reply.user_id).await?
                    .ok_or(anyhow::anyhow!("User not found"))?;
                let reply_author_role = reply_author.role.as_str().to_string();
                let reply_avatar = self.user_avatar_url(reply.user_id).await?;

                replies_with_authors.push(ReplyWithAuthor {
                    reply,
                    author_name: reply_author.full_name,
                    author_role: reply_author_role,
                    author_avatar: reply_avatar,
                });
            }

            result.push(CommentWithReplies {
                comment: comment_with_author,
                replies: replies_with_authors,
            });
        }

        Ok(result)
    }

    pub async fn get_comment_count(&self, property_id: Uuid) -> Result<i64> {
        self.repo.get_comment_count(property_id).await
    }
}
