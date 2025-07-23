use std::env;
use anyhow::Error;
use async_trait::async_trait;
use redis::{AsyncCommands, Client, FromRedisValue, SetExpiry, SetOptions, ToRedisArgs};
use crate::temp_store::TempStore;

pub struct Redis<K, V> where K: ToRedisArgs + Send + Sync, V: FromRedisValue + Send + Sync {
    client: Client,
}

#[async_trait]
impl<K, V> TempStore for Redis<K, V> where K: ToRedisArgs + Send + Sync, V: FromRedisValue + Send + Sync {
    type Key = K;
    type Value = V;

    fn new() -> Self {
        let url = env::var("REDIS_URL").expect("REDIS_URL must be set");
        let client = Client::open(url).expect("failed to open redis client");
        Self { client }
    }

    async fn get(&self, key: K) -> Option<V> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .ok()?
            .get(key)
            .await
            .ok()?
    }

    async fn set(&self, key: K, value: V) -> Result<(), Error> {
        self.client
            .get_multiplexed_async_connection()
            .await?
            .set_options(
                key,
                value,
                SetOptions::default().with_expiration(SetExpiry::EX(86400)),
            )
            .await.into()
    }

    async fn delete(&self, key: Self::Key)  -> Result<(), Error> {
        self.client
            .get_multiplexed_async_connection()
            .await?
            .del(key)
            .await.into()
    }
}