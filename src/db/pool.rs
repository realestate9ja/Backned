use anyhow::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str, max_connections: u32) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .max_lifetime(Duration::from_secs(1800))
        .idle_timeout(Duration::from_secs(600))
        .connect(database_url)
        .await
        .context("failed to connect to postgres")
}
