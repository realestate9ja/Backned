use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::Mutex;

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
    entries: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
}

impl RateLimiter {
    pub fn new(
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
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn check(&self, scope: RateLimitScope, key: &str) -> bool {
        let (max_requests, window) = match scope {
            RateLimitScope::Auth => (self.auth_max_requests, self.auth_window),
            RateLimitScope::Trust => (self.trust_max_requests, self.trust_window),
        };
        let now = Instant::now();
        let mut entries = self.entries.lock().await;
        let queue = entries.entry(format!("{}:{key}", scope_key(scope))).or_default();

        while let Some(front) = queue.front() {
            if now.duration_since(*front) > window {
                queue.pop_front();
            } else {
                break;
            }
        }

        if queue.len() >= max_requests {
            return false;
        }

        queue.push_back(now);
        true
    }
}

fn scope_key(scope: RateLimitScope) -> &'static str {
    match scope {
        RateLimitScope::Auth => "auth",
        RateLimitScope::Trust => "trust",
    }
}
