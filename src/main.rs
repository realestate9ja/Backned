mod application;
mod config;
mod db;
mod domain;
mod infrastructure;
mod interfaces;
mod utils;

use anyhow::Context;
use axum::Router;
use config::Settings;
use db::{create_pool, run_migrations};
use interfaces::http::{routes::create_router, state::AppState};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let settings = Settings::from_env()?;
    let pool = create_pool(&settings.database_url, settings.database_max_connections).await?;
    run_migrations(&pool).await?;
    let state = AppState::new(pool, settings.clone());
    let app: Router = create_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], settings.port));
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind to {addr}"))?;
    let local_addr = listener.local_addr().context("failed to read local socket address")?;
    tracing::info!("verinest backend listening on {}", local_addr);

    axum::serve(listener, app)
        .await
        .context("server failed")?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "realestate=debug,tower_http=info,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
