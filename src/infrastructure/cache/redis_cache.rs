use anyhow::Result;
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
pub struct CacheService {
    client: Client,
    ttl_seconds: u64,
}

impl CacheService {
    pub fn new(redis_url: &str, ttl_seconds: u64) -> Result<Self> {
        Ok(Self {
            client: Client::open(redis_url)?,
            ttl_seconds,
        })
    }

    pub async fn get_json<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let mut connection = self.connection().await?;
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
        let mut connection = self.connection().await?;
        let payload = serde_json::to_string(value)?;
        let _: () = connection.set_ex(key, payload, self.ttl_seconds).await?;
        Ok(())
    }

    pub async fn invalidate_namespace(&self, namespace: &str) -> Result<()> {
        let mut connection = self.connection().await?;
        let _: i64 = connection.incr(self.namespace_version_key(namespace), 1).await?;
        Ok(())
    }

    pub async fn versioned_key(&self, namespace: &str, suffix: &str) -> Result<String> {
        let version_key = self.namespace_version_key(namespace);
        let mut connection = self.connection().await?;
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

    async fn connection(&self) -> Result<ConnectionManager> {
        Ok(ConnectionManager::new(self.client.clone()).await?)
    }

    fn namespace_version_key(&self, namespace: &str) -> String {
        format!("verinest:{namespace}:version")
    }
}
