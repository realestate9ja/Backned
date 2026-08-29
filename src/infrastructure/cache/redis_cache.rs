use anyhow::Result;
use redis::{AsyncCommands, Client, aio::ConnectionManager};
use serde::{Serialize, de::DeserializeOwned};
use tokio::time::{timeout, Duration};
use tracing::warn;

#[derive(Clone)]
pub struct CacheService {
    connection: Option<ConnectionManager>,
    ttl_seconds: u64,
}

impl CacheService {
    pub async fn new(redis_url: &str, ttl_seconds: u64) -> Self {
        let connection = match Client::open(redis_url) {
            Ok(client) => match timeout(Duration::from_secs(3), ConnectionManager::new(client)).await {
                Ok(Ok(connection)) => Some(connection),
                Ok(Err(error)) => {
                    warn!(
                        error = %error,
                        "redis cache disabled; falling back to no-op cache for local startup"
                    );
                    None
                }
                Err(_) => {
                    warn!(
                        "redis cache connection timed out; falling back to no-op cache for local startup"
                    );
                    None
                }
            },
            Err(error) => {
                warn!(
                    error = %error,
                    "redis cache disabled; falling back to no-op cache for local startup"
                );
                None
            }
        };
        Self {
            connection,
            ttl_seconds,
        }
    }

    pub async fn get_json<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let Some(mut connection) = self.connection.as_ref().cloned() else {
            return Ok(None);
        };
        let payload: Option<String> = connection.get(key).await?;
        match payload {
            Some(payload) => Ok(Some(serde_json::from_str(&payload)?)),
            None => Ok(None),
        }
    }

    pub async fn set_json<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let Some(mut connection) = self.connection.as_ref().cloned() else {
            return Ok(());
        };
        let payload = serde_json::to_string(value)?;
        let _: () = connection.set_ex(key, payload, self.ttl_seconds).await?;
        Ok(())
    }

    pub async fn invalidate_namespace(&self, namespace: &str) -> Result<()> {
        let Some(mut connection) = self.connection.as_ref().cloned() else {
            return Ok(());
        };
        let _: i64 = connection
            .incr(self.namespace_version_key(namespace), 1)
            .await?;
        Ok(())
    }

    pub async fn versioned_key(&self, namespace: &str, suffix: &str) -> Result<String> {
        if self.connection.is_none() {
            return Ok(format!("verinest:{namespace}:local:{suffix}"));
        }
        let version_key = self.namespace_version_key(namespace);
        let mut connection = self.connection();
        let version: Option<u64> = connection.get(&version_key).await?;
        let version = match version {
            Some(version) => version,
            None => {
                let _: bool = connection.set_nx(&version_key, 1_u64).await?;
                1
            }
        };
        Ok(format!("verinest:{namespace}:v{version}:{suffix}"))
    }

    pub fn connection(&self) -> ConnectionManager {
        self.connection
            .as_ref()
            .expect("redis connection not available")
            .clone()
    }

    pub fn maybe_connection(&self) -> Option<ConnectionManager> {
        self.connection.clone()
    }

    fn namespace_version_key(&self, namespace: &str) -> String {
        format!("verinest:{namespace}:version")
    }
}
