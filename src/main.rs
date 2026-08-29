use anyhow::Context;
use axum::Router;
use realestate::{
    application::services::BookingEmailReminderService,
    application::services::VerificationReminderService,
    build_app_with_state,
    config::Settings,
    db::{create_pool, run_migrations},
    interfaces::http::state::AppState,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let settings = Settings::from_env()?;
    let pool = create_pool(&settings.database_url, settings.database_max_connections).await?;
    if settings.run_migrations_on_startup {
        run_migrations(&pool).await?;
        tracing::info!("database migrations executed during startup");
    } else {
        tracing::info!("database migrations skipped during startup");
    }
    let read_pool = if let Some(read_database_url) = &settings.read_database_url {
        create_pool(read_database_url, settings.database_max_connections).await?
    } else {
        pool.clone()
    };
    let state = AppState::new(pool, read_pool, settings.clone()).await;
    if settings.run_booking_reminder_cron {
        BookingEmailReminderService::new(
            state.pool.clone(),
            state.mail_service.clone(),
            settings.app_base_url.clone(),
        )
        .spawn_cron();
        tracing::info!("booking reminder cron enabled in API process");
    } else {
        tracing::info!("booking reminder cron disabled in API process");
    }
    if settings.run_verification_reminder_cron {
        VerificationReminderService::new(
            state.pool.clone(),
            state.mail_service.clone(),
            settings.app_base_url.clone(),
        )
        .spawn_cron();
        tracing::info!("verification reminder cron enabled in API process");
    } else {
        tracing::info!("verification reminder cron disabled in API process");
    }
    let app: Router = build_app_with_state(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], settings.port));
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind to {addr}"))?;
    let local_addr = listener
        .local_addr()
        .context("failed to read local socket address")?;
    tracing::info!("verinest backend listening on {}", local_addr);

    axum::serve(listener, app).await.context("server failed")?;

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
