use async_trait::async_trait;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[derive(Error, Debug)]
#[error("Error in cache operation")]
pub struct CacheError(String);

impl CacheError {
    #[must_use]
    pub fn new(msg: String) -> Self {
        Self(msg)
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Cache {
    fn create() -> Self;
    async fn get(&self, key: String) -> Option<String>;
    async fn set(&self, key: String, value: String) -> Result<(), CacheError>;
    async fn delete(&self, key: String) -> Result<(), CacheError>;
}
