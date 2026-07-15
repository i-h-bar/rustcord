use async_trait::async_trait;
use serenity::builder::CreateEmbed;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("webhook request failed: {0}")]
    Request(String),
    #[error("webhook returned non-success status {0}")]
    Status(u16),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait WebhookSender {
    async fn send(
        &self,
        webhook_id: u64,
        webhook_token: &str,
        embed: CreateEmbed,
    ) -> Result<(), WebhookError>;
}
