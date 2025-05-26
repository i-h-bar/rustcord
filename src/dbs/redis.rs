use redis::{AsyncCommands, Client, FromRedisValue, SetExpiry, SetOptions, ToRedisArgs};
use std::env;
use std::sync::LazyLock;

pub static REDIS: LazyLock<Redis> = LazyLock::new(Redis::new);

#[derive(Debug)]
pub struct Redis {
    client: Client,
}

impl Redis {
    fn new() -> Self {
        let url = env::var("REDIS_URL").expect("REDIS_URL must be set");
        let client = Client::open(url).expect("failed to open redis client");
        Self { client }
    }

    pub async fn get<K: ToRedisArgs + Send + Sync, V: FromRedisValue + Send + Sync>(
        &self,
        key: K,
    ) -> Option<V> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .ok()?
            .get(key)
            .await
            .ok()?
    }

    pub async fn set<K: ToRedisArgs + Send + Sync, V: ToRedisArgs + Send + Sync>(
        &self,
        key: K,
        value: V,
    ) -> Result<(), redis::RedisError> {
        self.client
            .get_multiplexed_async_connection()
            .await?
            .set_options(
                key,
                value,
                SetOptions::default().with_expiration(SetExpiry::EX(86400)),
            )
            .await
    }

    pub async fn delete<K: ToRedisArgs + Send + Sync>(
        &self,
        key: K,
    ) -> Result<(), redis::RedisError> {
        self.client
            .get_multiplexed_async_connection()
            .await?
            .del(key)
            .await
    }
}
