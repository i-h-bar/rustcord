use async_trait::async_trait;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[derive(Debug, Error)]
#[error("webhook operation failed: {0}")]
pub struct SpoilerSubError(String);

impl SpoilerSubError {
    #[must_use]
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

pub struct Subscription {
    pub id: i64,
    pub token: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SpoilerSubscription {
    fn create() -> Self;
    async fn create_subscription(&self, channel_id: u64)
        -> Result<Subscription, SpoilerSubError>;
    async fn delete_subscription(&self, webhook_id: u64) -> Result<(), SpoilerSubError>;
}
