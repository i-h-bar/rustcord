use async_trait::async_trait;
use contracts::card::Card;
use thiserror::Error;

use cards_sdk::SubscriptionId;
#[cfg(test)]
use mockall::automock;

#[derive(Debug, Error)]
pub enum SpoilerError {
    #[error("Spoiler request failed: {0}")]
    Request(String),
    #[error("Spoiler route returned non-success status {0}")]
    Status(u16),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SpoilerSender {
    /// Sends every `(card, image bytes)` pair as a single message — Discord
    /// allows up to 10 embeds and attachments per webhook execution, and
    /// `notify::run` never stages more than `BATCH_SIZE` (10) cards at once,
    /// so one call always covers a full batch. This is all-or-nothing: on
    /// failure none of the batch is acked and the whole thing is retried
    /// next poll, trading per-card retry granularity for one Discord API
    /// call per subscription per poll instead of one per card.
    async fn send(
        &self,
        sub_id: SubscriptionId,
        token: &str,
        cards: &[(Card, Vec<u8>)],
    ) -> Result<(), SpoilerError>;
}
