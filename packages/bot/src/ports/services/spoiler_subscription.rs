use async_trait::async_trait;
use cards_sdk::{ChannelId, SubscriptionId};
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[derive(Debug, Error)]
#[error("webhook operation failed: {message}")]
pub struct SpoilerSubError {
    message: String,
    status_code: Option<u16>,
}

impl SpoilerSubError {
    #[must_use]
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            status_code: None,
        }
    }

    #[must_use]
    pub fn with_status(msg: impl Into<String>, status_code: u16) -> Self {
        Self {
            message: msg.into(),
            status_code: Some(status_code),
        }
    }

    /// Whether Discord reported this operation as a 404. For
    /// `delete_subscription`, that means the webhook was already gone (e.g.
    /// manually removed) — the caller should treat that as a successful
    /// deletion rather than a transient failure worth keeping the
    /// subscription record around for.
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        self.status_code == Some(404)
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
