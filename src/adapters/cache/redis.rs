use crate::adapters::cache::{Cache, CacheError};
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, SetExpiry, SetOptions};
use std::env;

pub struct Redis {
    client: Client,
}

#[async_trait]
impl Cache for Redis {
    fn new() -> Self {
        let url = env::var("REDIS_URL").expect("REDIS_URL must be set");
        log::info!("Using redis cache: {}", url);
        let client = Client::open(url).expect("failed to open redis client");
        Self { client }
    }

    async fn get(&self, key: String) -> Option<String> {
        match self.new_connection().await.ok()?.get(key).await {
            Ok(value) => Some(value),
            Err(why) => {
                log::warn!("Error getting value in cache {why:?}");
                None
            }
        }
    }

    async fn set(&self, key: String, value: String) -> Result<(), CacheError> {
        match self
            .new_connection()
            .await?
            .set_options::<String, String, ()>(
                key,
                value,
                SetOptions::default().with_expiration(SetExpiry::EX(86400)),
            )
            .await
        {
            Ok(()) => Ok(()),
            Err(why) => {
                log::warn!("Error setting value in cache {why:?}");
                Err(CacheError(String::from("Unable to set value")))
            }
        }
    }

    async fn delete(&self, key: String) -> Result<(), CacheError> {
        match self.new_connection().await?.del(key).await {
            Ok(()) => Ok(()),
            Err(why) => {
                log::warn!("Error deleting value in cache {why:?}");
                Err(CacheError(String::from("Unable to delete cache")))
            }
        }
    }
}

impl Redis {
    async fn new_connection(&self) -> Result<MultiplexedConnection, CacheError> {
        match self.client.get_multiplexed_async_connection().await {
            Ok(connection) => Ok(connection),
            Err(why) => {
                log::warn!("Error making connection {why:?}");
                Err(CacheError(String::from("Unable to get connection")))
            }
        }
    }
}
