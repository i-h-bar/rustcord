use crate::spi::cache::{Cache, CacheError};
use async_trait::async_trait;
use redis::{AsyncCommands, Client, SetExpiry, SetOptions};
use std::env;

pub struct Redis {
    client: Client,
}

#[async_trait]
impl Cache for Redis {
    fn new() -> Self {
        let url = env::var("REDIS_URL").expect("REDIS_URL must be set");
        let client = Client::open(url).expect("failed to open redis client");
        Self { client }
    }

    async fn get(&self, key: String) -> Option<String> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .ok()?
            .get(key)
            .await
            .ok()?
    }

    async fn set(&self, key: String, value: String) -> Result<(), CacheError> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| CacheError(String::from("Unable to get connection")))?
            .set_options(
                key,
                value,
                SetOptions::default().with_expiration(SetExpiry::EX(86400)),
            )
            .await
            .map_err(|_| CacheError(String::from("Unable to set value")))
    }

    async fn delete(&self, key: String) -> Result<(), CacheError> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| CacheError(String::from("Unable to get connection")))?
            .del(key)
            .await
            .map_err(|_| CacheError(String::from("Unable to delete key")))
    }
}
