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

        Ok(Self {
            database_url,
            redis_url,
            cache_ttl_seconds,
            jwt_secret,
            jwt_expiration_minutes,
            port,
            database_max_connections,
        })
    }
}
