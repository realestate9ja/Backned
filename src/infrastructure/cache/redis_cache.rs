use anyhow::Result;
use redis::{AsyncCommands, Client, aio::ConnectionManager};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct CacheService {
    connection: ConnectionManager,
    ttl_seconds: u64,
}

impl CacheService {
    pub async fn new(redis_url: &str, ttl_seconds: u64) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;
        Ok(Self {
            connection,
            ttl_seconds,
        })
    }

    pub async fn get_json<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let mut connection = self.connection();
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
        let mut connection = self.connection();
        let payload = serde_json::to_string(value)?;
        let _: () = connection.set_ex(key, payload, self.ttl_seconds).await?;
        Ok(())
    }

    pub async fn invalidate_namespace(&self, namespace: &str) -> Result<()> {
        let mut connection = self.connection();
        let _: i64 = connection
            .incr(self.namespace_version_key(namespace), 1)
            .await?;
        Ok(())
    }

    pub async fn versioned_key(&self, namespace: &str, suffix: &str) -> Result<String> {
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
        self.connection.clone()
    }

    fn namespace_version_key(&self, namespace: &str) -> String {
        format!("verinest:{namespace}:version")
    }
}
