use anyhow::Context;
use axum::Router;
use realestate::{build_app, config::Settings, db::{create_pool, run_migrations}};
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
    let app: Router = build_app(pool, settings.clone());
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
