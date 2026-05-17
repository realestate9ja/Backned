use std::time::Duration;

use anyhow::Result;
use redis::{Script, aio::ConnectionManager};

#[derive(Clone, Copy)]
pub enum RateLimitScope {
    Auth,
    Trust,
}

#[derive(Clone)]
pub struct RateLimiter {
    auth_max_requests: usize,
    auth_window: Duration,
    trust_max_requests: usize,
    trust_window: Duration,
    connection: ConnectionManager,
}

impl RateLimiter {
    pub fn new(
        connection: ConnectionManager,
        auth_max_requests: usize,
        auth_window_seconds: u64,
        trust_max_requests: usize,
        trust_window_seconds: u64,
    ) -> Self {
        Self {
            auth_max_requests,
            auth_window: Duration::from_secs(auth_window_seconds),
            trust_max_requests,
            trust_window: Duration::from_secs(trust_window_seconds),
            connection,
        }
    }

    pub async fn check(&self, scope: RateLimitScope, key: &str) -> Result<bool> {
        let (max_requests, window) = match scope {
            RateLimitScope::Auth => (self.auth_max_requests, self.auth_window),
            RateLimitScope::Trust => (self.trust_max_requests, self.trust_window),
        };

        let redis_key = format!("verinest:rate_limit:{}:{key}", scope_key(scope));
        let script = Script::new(
            r#"
            local current = redis.call("INCR", KEYS[1])
            if current == 1 then
                redis.call("EXPIRE", KEYS[1], ARGV[1])
            end
            return current
            "#,
        );

        let mut connection = self.connection.clone();
        let current: i64 = script
            .key(redis_key)
            .arg(window.as_secs() as i64)
            .invoke_async(&mut connection)
            .await?;
        Ok(current <= max_requests as i64)
    }
}

fn scope_key(scope: RateLimitScope) -> &'static str {
    match scope {
        RateLimitScope::Auth => "auth",
        RateLimitScope::Trust => "trust",
    }
}
