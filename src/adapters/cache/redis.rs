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
    fn create() -> Self {
        let url = env::var("REDIS_URL").expect("REDIS_URL must be set");
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
        if let Err(why) = self
            .new_connection()
            .await?
            .set_options::<String, String, ()>(
                key,
                value,
                SetOptions::default().with_expiration(SetExpiry::EX(86400)),
            )
            .await
        {
            log::warn!("Error setting value in cache {why:?}");
            Err(CacheError(String::from("Unable to set value")))
        } else {
            Ok(())
        }
    }

    async fn delete(&self, key: String) -> Result<(), CacheError> {
        if let Err(why) = self.new_connection().await?.del::<String, ()>(key).await {
            log::warn!("Error deleting value in cache {why:?}");
            Err(CacheError(String::from("Unable to delete cache")))
        } else {
            Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_error_display() {
        let error = CacheError(String::from("Test error"));
        assert_eq!(error.to_string(), "Error in cache operation");
    }

    #[test]
    fn test_cache_error_debug() {
        let error = CacheError(String::from("Test error"));
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("CacheError"));
    }
}
