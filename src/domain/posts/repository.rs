use crate::domain::posts::{CreatePostInput, Post, PostListItem};
use anyhow::Result;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostRepository {
    pool: PgPool,
}

impl PostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: &CreatePostInput, author_id: Uuid) -> Result<Post> {
        let post = sqlx::query_as::<_, Post>(
            r#"
            INSERT INTO posts (id, author_id, budget, location, city, state, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, author_id, budget, location, city, state, description, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(author_id)
        .bind(input.budget)
        .bind(&input.location)
        .bind(&input.city)
        .bind(&input.state)
        .bind(&input.description)
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }

    pub async fn list(
        &self,
        limit: i64,
        offset: i64,
        location: Option<&str>,
        city: Option<&str>,
        state: Option<&str>,
        min_budget: Option<i64>,
        max_budget: Option<i64>,
    ) -> Result<Vec<PostListItem>> {
        let mut builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                p.id,
                p.author_id,
                u.full_name AS author_name,
                u.role::text AS author_role,
                p.budget,
                p.location,
                p.city,
                p.state,
                p.description,
                COUNT(r.id)::bigint AS response_count,
                p.created_at
            FROM posts p
            INNER JOIN users u ON u.id = p.author_id
            LEFT JOIN responses r ON r.post_id = p.id
            WHERE 1 = 1
            "#,
        );

        if let Some(location) = location {
            builder.push(" AND p.location ILIKE ");
            builder.push_bind(format!("%{location}%"));
        }
        if let Some(city) = city {
            builder.push(" AND p.city ILIKE ");
            builder.push_bind(format!("%{city}%"));
        }
        if let Some(state) = state {
            builder.push(" AND p.state ILIKE ");
            builder.push_bind(format!("%{state}%"));
        }
        if let Some(min_budget) = min_budget {
            builder.push(" AND p.budget >= ");
            builder.push_bind(min_budget);
        }
        if let Some(max_budget) = max_budget {
            builder.push(" AND p.budget <= ");
            builder.push_bind(max_budget);
        }

        builder.push(
            r#"
            GROUP BY p.id, u.full_name, u.role
            ORDER BY p.created_at DESC
            LIMIT
            "#,
        );
        builder.push_bind(limit);
        builder.push(" OFFSET ");
        builder.push_bind(offset);

        let posts = builder
            .build_query_as::<PostListItem>()
            .fetch_all(&self.pool)
            .await?;

        Ok(posts)
    }

    pub async fn list_recent_by_author(&self, author_id: Uuid, limit: i64) -> Result<Vec<PostListItem>> {
        let posts = sqlx::query_as::<_, PostListItem>(
            r#"
            SELECT
                p.id,
                p.author_id,
                u.full_name AS author_name,
                u.role::text AS author_role,
                p.budget,
                p.location,
                p.city,
                p.state,
                p.description,
                COUNT(r.id)::bigint AS response_count,
                p.created_at
            FROM posts p
            INNER JOIN users u ON u.id = p.author_id
            LEFT JOIN responses r ON r.post_id = p.id
            WHERE p.author_id = $1
            GROUP BY p.id, u.full_name, u.role
            ORDER BY p.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(author_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(posts)
    }

    pub async fn count_by_author(&self, author_id: Uuid) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)::bigint FROM posts WHERE author_id = $1",
        )
        .bind(author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn exists(&self, post_id: Uuid) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM posts WHERE id = $1)")
            .bind(post_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(exists)
    }
}
