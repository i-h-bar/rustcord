use async_trait::async_trait;
use cards_sdk::{ChannelId, SubscriptionId};
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
    pub id: SubscriptionId,
    pub token: String,
}

/// A source of spoiler-announcement subscriptions — currently always
/// Discord webhooks (`DiscordWebhookRegistrar`), but this trait deliberately
/// speaks in `cards_sdk`'s platform-agnostic `ChannelId`/`SubscriptionId`
/// newtypes rather than Discord's native `u64` snowflakes, so a future
/// non-Discord implementation isn't forced to shoehorn its own identifiers
/// into Discord's representation. Only the concrete Discord adapter
/// (`adapters/services/spoiler_subscription/discord.rs`) ever touches a raw
/// `u64`.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait SpoilerSubscription {
    fn create() -> Self;
    async fn create_subscription(
        &self,
        channel_id: ChannelId,
    ) -> Result<Subscription, SpoilerSubError>;
    async fn delete_subscription(&self, sub_id: SubscriptionId) -> Result<(), SpoilerSubError>;
}
