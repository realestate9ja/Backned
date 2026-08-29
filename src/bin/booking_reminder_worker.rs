use anyhow::Context;
use realestate::{
    application::services::BookingEmailReminderService,
    config::Settings,
    db::{create_pool, run_migrations},
    infrastructure::email::MailService,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let settings = Settings::from_env()?;
    let pool = create_pool(&settings.database_url, settings.database_max_connections).await?;
    if settings.run_migrations_on_startup {
        run_migrations(&pool).await?;
        tracing::info!("database migrations executed during worker startup");
    } else {
        tracing::info!("database migrations skipped during worker startup");
    }

    let mail_service = build_mail_service(&settings)?;
    BookingEmailReminderService::new(pool, mail_service, settings.app_base_url.clone()).spawn_cron();

    tracing::info!("booking reminder worker started");
    tokio::signal::ctrl_c()
        .await
        .context("booking reminder worker failed while waiting for shutdown signal")?;
    tracing::info!("booking reminder worker shutting down");
    Ok(())
}

fn build_mail_service(settings: &Settings) -> anyhow::Result<MailService> {
    match settings.mail_provider.trim().to_lowercase().as_str() {
        "disabled" => Ok(MailService::disabled(
            settings.mail_from_email.clone(),
            settings.mail_from_name.clone(),
        )),
        "resend" => Ok(MailService::resend(
            settings.mail_from_email.clone(),
            settings.mail_from_name.clone(),
            settings.resend_api_key.clone().ok_or_else(|| {
                anyhow::anyhow!("RESEND_API_KEY is required when MAIL_PROVIDER=resend")
            })?,
        )),
        "smtp" => MailService::smtp(
            settings.mail_from_email.clone(),
            settings.mail_from_name.clone(),
            settings
                .smtp_host
                .clone()
                .ok_or_else(|| anyhow::anyhow!("SMTP_HOST is required when MAIL_PROVIDER=smtp"))?,
            settings.smtp_port,
            settings.smtp_username.clone().ok_or_else(|| {
                anyhow::anyhow!("SMTP_USERNAME is required when MAIL_PROVIDER=smtp")
            })?,
            settings.smtp_password.clone().ok_or_else(|| {
                anyhow::anyhow!("SMTP_PASSWORD is required when MAIL_PROVIDER=smtp")
            })?,
            settings.smtp_use_starttls,
        ),
        provider => Err(anyhow::anyhow!("unsupported MAIL_PROVIDER: {provider}")),
    }
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
