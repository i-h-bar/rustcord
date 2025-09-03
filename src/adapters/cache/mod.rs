mod redis;

use crate::adapters::cache::redis::Redis;
use async_trait::async_trait;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[derive(Error, Debug)]
#[error("Error in cache operation")]
pub struct CacheError(String);

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Cache {
    fn new() -> Self;
    async fn get(&self, key: String) -> Option<String>;
    async fn set(&self, key: String, value: String) -> Result<(), CacheError>;
    async fn delete(&self, key: String) -> Result<(), CacheError>;
}

pub async fn init_cache() -> impl Cache {
    Redis::new()
}
