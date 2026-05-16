use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PropertyComment {
    pub id: Uuid,
    pub property_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CommentReply {
    pub id: Uuid,
    pub comment_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentWithAuthor {
    pub comment: PropertyComment,
    pub author_name: String,
    pub author_role: String,
    pub author_avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentWithReplies {
    pub comment: CommentWithAuthor,
    pub replies: Vec<ReplyWithAuthor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyWithAuthor {
    pub reply: CommentReply,
    pub author_name: String,
    pub author_role: String,
    pub author_avatar: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CommentWithAuthorRow {
    pub id: Uuid,
    pub property_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub author_name: String,
    pub author_role: String,
    pub author_avatar: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReplyWithAuthorRow {
    pub id: Uuid,
    pub comment_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub author_name: String,
    pub author_role: String,
    pub author_avatar: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentInput {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateReplyInput {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentInput {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReplyInput {
    pub content: String,
}
