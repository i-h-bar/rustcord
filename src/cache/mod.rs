mod redis;

use async_trait::async_trait;
use thiserror::Error;
use crate::cache::redis::Redis;

#[derive(Error, Debug)]
#[error("Error in cache operation")]
pub struct CacheError(String);


#[async_trait]
pub trait Cache {
    fn new() -> Self;
    async fn get(&self, key: String) -> Option<String>;
    async fn set(&self, key: String, value: String) -> Result<(), CacheError>;
    async fn delete(&self, key: String)  -> Result<(), CacheError>;
}

pub async fn init_cache() -> impl Cache {
    Redis::new()
}