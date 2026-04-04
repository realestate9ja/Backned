use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Settings {
    pub database_url: String,
    pub redis_url: String,
    pub cache_ttl_seconds: u64,
    pub jwt_secret: String,
    pub jwt_expiration_minutes: i64,
    pub port: u16,
    pub database_max_connections: u32,
    pub admin_bootstrap_token: String,
    pub livekit_url: String,
    pub livekit_api_key: String,
    pub livekit_api_secret: String,
    pub livekit_token_ttl_minutes: i64,
    pub auth_rate_limit_max_requests: usize,
    pub auth_rate_limit_window_seconds: u64,
    pub trust_rate_limit_max_requests: usize,
    pub trust_rate_limit_window_seconds: u64,
    pub app_base_url: String,
    pub mail_provider: String,
    pub mail_from_email: String,
    pub mail_from_name: String,
    pub resend_api_key: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_use_starttls: bool,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL is required")?;
        let redis_url = std::env::var("REDIS_URL").context("REDIS_URL is required")?;
        let cache_ttl_seconds = std::env::var("CACHE_TTL_SECONDS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .context("CACHE_TTL_SECONDS must be a valid integer")?;
        let jwt_secret = std::env::var("JWT_SECRET").context("JWT_SECRET is required")?;
        let jwt_expiration_minutes = std::env::var("JWT_EXPIRATION_MINUTES")
            .unwrap_or_else(|_| "1440".to_string())
            .parse()
            .context("JWT_EXPIRATION_MINUTES must be a valid integer")?;
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .context("PORT must be a valid integer")?;
        let database_max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .context("DATABASE_MAX_CONNECTIONS must be a valid integer")?;
        let admin_bootstrap_token =
            std::env::var("ADMIN_BOOTSTRAP_TOKEN").unwrap_or_else(|_| "dev-admin-bootstrap-token".to_string());
        let livekit_url =
            std::env::var("LIVEKIT_URL").unwrap_or_else(|_| "ws://127.0.0.1:7880".to_string());
        let livekit_api_key =
            std::env::var("LIVEKIT_API_KEY").unwrap_or_else(|_| "dev-livekit-key".to_string());
        let livekit_api_secret =
            std::env::var("LIVEKIT_API_SECRET").unwrap_or_else(|_| "dev-livekit-secret".to_string());
        let livekit_token_ttl_minutes = std::env::var("LIVEKIT_TOKEN_TTL_MINUTES")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .context("LIVEKIT_TOKEN_TTL_MINUTES must be a valid integer")?;
        let auth_rate_limit_max_requests = std::env::var("AUTH_RATE_LIMIT_MAX_REQUESTS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .context("AUTH_RATE_LIMIT_MAX_REQUESTS must be a valid integer")?;
        let auth_rate_limit_window_seconds = std::env::var("AUTH_RATE_LIMIT_WINDOW_SECONDS")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .context("AUTH_RATE_LIMIT_WINDOW_SECONDS must be a valid integer")?;
        let trust_rate_limit_max_requests = std::env::var("TRUST_RATE_LIMIT_MAX_REQUESTS")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .context("TRUST_RATE_LIMIT_MAX_REQUESTS must be a valid integer")?;
        let trust_rate_limit_window_seconds = std::env::var("TRUST_RATE_LIMIT_WINDOW_SECONDS")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .context("TRUST_RATE_LIMIT_WINDOW_SECONDS must be a valid integer")?;
        let app_base_url = std::env::var("APP_BASE_URL")
            .unwrap_or_else(|_| format!("http://127.0.0.1:{port}"));
        let mail_provider = std::env::var("MAIL_PROVIDER").unwrap_or_else(|_| "disabled".to_string());
        let mail_from_email =
            std::env::var("MAIL_FROM_EMAIL").unwrap_or_else(|_| "noreply@verinest.local".to_string());
        let mail_from_name =
            std::env::var("MAIL_FROM_NAME").unwrap_or_else(|_| "VeriNest".to_string());
        let resend_api_key = std::env::var("RESEND_API_KEY").ok().filter(|value| !value.trim().is_empty());
        let smtp_host = std::env::var("SMTP_HOST").ok().filter(|value| !value.trim().is_empty());
        let smtp_port = std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .context("SMTP_PORT must be a valid integer")?;
        let smtp_username = std::env::var("SMTP_USERNAME").ok().filter(|value| !value.trim().is_empty());
        let smtp_password = std::env::var("SMTP_PASSWORD").ok().filter(|value| !value.trim().is_empty());
        let smtp_use_starttls = std::env::var("SMTP_USE_STARTTLS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .context("SMTP_USE_STARTTLS must be a valid boolean")?;

        Ok(Self {
            database_url,
            redis_url,
            cache_ttl_seconds,
            jwt_secret,
            jwt_expiration_minutes,
            port,
            database_max_connections,
            admin_bootstrap_token,
            livekit_url,
            livekit_api_key,
            livekit_api_secret,
            livekit_token_ttl_minutes,
            auth_rate_limit_max_requests,
            auth_rate_limit_window_seconds,
            trust_rate_limit_max_requests,
            trust_rate_limit_window_seconds,
            app_base_url,
            mail_provider,
            mail_from_email,
            mail_from_name,
            resend_api_key,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            smtp_use_starttls,
        })
    }
}
