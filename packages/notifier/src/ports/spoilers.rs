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

impl SpoilerError {
    /// A 404 means Discord doesn't know this webhook at all (deleted,
    /// channel removed, etc.) — unambiguous and won't fix itself, so
    /// `notify::run` unsubscribes on the first occurrence rather than
    /// waiting for `MAX_CONSECUTIVE_FAILURES`.
    #[must_use]
    pub fn is_permanently_gone(&self) -> bool {
        matches!(self, SpoilerError::Status(404))
    }

    /// Whether this failure is specific to this one webhook (invalid/revoked
    /// token, missing permissions) rather than a sign Discord itself is
    /// having trouble. Connection errors, server errors (5xx), and rate
    /// limiting (429) are deliberately excluded — a Discord-wide outage
    /// would make *every* subscription fail at once, and those must never
    /// count toward auto-unsubscribing an otherwise-healthy subscription.
    #[must_use]
    pub fn counts_toward_failure_threshold(&self) -> bool {
        matches!(self, SpoilerError::Status(401 | 403))
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SpoilerSender {
    /// Sends one card as its own message — deliberately *not* batched into a
    /// single multi-embed message, so each card can have its own Discord
    /// thread started on it for discussion. `notify::run` sends these one at
    /// a time (never concurrently) for a given subscription, acking as far
    /// as it gets and stopping at the first failure, so a mid-batch failure
    /// never causes an already-delivered card to be resent next poll.
    async fn send(
        &self,
        sub_id: SubscriptionId,
        token: &str,
        card: &Card,
        image: Vec<u8>,
    ) -> Result<(), SpoilerError>;
}
