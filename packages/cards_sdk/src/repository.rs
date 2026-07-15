use crate::ingest::{CardInfo, UpsertResult};
use crate::spoiler::{PendingCard, Subscription};
use async_trait::async_trait;
use contracts::card::Card;
use contracts::card_set::CardSet;
use uuid::Uuid;

#[cfg(feature = "test-util")]
use mockall::automock;

#[cfg_attr(feature = "test-util", automock)]
#[async_trait]
pub trait ReadRepository {
    async fn search(&self, normalised_name: &str) -> Option<Vec<Card>>;
    async fn search_artist(&self, artist: &str, normalised_name: &str) -> Option<Vec<Card>>;
    async fn search_set(&self, set_name: &str, normalised_name: &str) -> Option<Vec<Card>>;
    async fn search_for_set_name(&self, normalised_name: &str) -> Option<Vec<String>>;
    async fn set_name_from_abbreviation(&self, abbreviation: &str) -> Option<String>;
    async fn random_card(&self) -> Option<Card>;
    async fn random_card_from_set(&self, set_name: &str) -> Option<Card>;
    async fn all_prints(&self, oracle_id: &Uuid) -> Option<Vec<CardSet>>;
    async fn fetch_card_by_id(&self, id: &Uuid) -> Option<Card>;
    async fn similar_cards(&self, card: &Card) -> Option<Vec<Card>>;
}

#[cfg_attr(feature = "test-util", automock)]
#[async_trait]
pub trait WriteRepository {
    async fn upsert_cards(&self, cards: &[CardInfo]) -> UpsertResult;
    async fn delete_orphaned_images(&self, ids: &[Uuid]) -> Vec<Uuid>;
    async fn delete_orphaned_illustrations(&self, ids: &[Uuid]) -> Vec<Uuid>;
}

#[cfg_attr(feature = "test-util", automock)]
#[async_trait]
pub trait SpoilerQueue {
    async fn subscriptions_with_pending(&self) -> Vec<Subscription>;
    async fn pending_cards(&self, guild_id: i64, limit: i64) -> Vec<PendingCard>;
    /// Advances the cursor and resets `consecutive_failures` to `0` — acking
    /// only ever follows a successful delivery (see `notifier`'s
    /// `domain::notify::run`), so this is where the failure streak clears.
    async fn ack(&self, guild_id: i64, up_to_queue_id: i64);
    async fn create_subscription(
        &self,
        guild_id: i64,
        channel_id: i64,
        webhook_id: i64,
        webhook_token: &str,
    );
    /// Deletes the subscription and returns the `webhook_id` it had, if it
    /// existed — the caller (`bot`'s `/spoilers unsubscribe` handler) needs
    /// this to also delete the actual Discord webhook via `WebhookRegistrar`,
    /// since `cards_sdk` has no Discord API access of its own.
    async fn delete_subscription(&self, guild_id: i64) -> Option<i64>;
    /// Increments `consecutive_failures` for a guild whose webhook delivery
    /// just failed and returns the new count (`0` if the subscription no
    /// longer exists, e.g. a concurrent unsubscribe raced this call — safe
    /// default, since there's nothing left to auto-unsubscribe). `notifier`
    /// compares the returned count against its own failure-threshold
    /// constant and decides whether to call `delete_subscription` — the
    /// threshold is delivery policy, not something `cards_sdk` should own,
    /// matching this crate's existing "repository stays dumb data access"
    /// principle.
    async fn record_failure(&self, guild_id: i64) -> i64;
    /// Deletes `spoiler_queue` rows that are no longer useful, for either of
    /// two reasons: every current subscription has already passed them (i.e.
    /// below the *minimum* cursor across all subscriptions — a row still
    /// needed by even one lagging-but-healthy guild is never touched this
    /// way), or they're older than a fixed age cap regardless of delivery
    /// state (a safety valve for a subscription whose webhook is silently
    /// dead and will never advance its cursor, which would otherwise block
    /// the first condition forever). Called once per `notifier` run, after
    /// that run's delivery loop finishes advancing cursors — see
    /// `notifier`'s `domain::notify::run`.
    async fn prune_queue(&self);
}

#[cfg(all(test, feature = "test-util"))]
mod tests {
    use super::*;
    use mockall::predicate::eq;
    use uuid::uuid;

    #[tokio::test]
    async fn mock_read_repository_satisfies_the_trait() {
        let id = uuid!("00000000-0000-0000-0000-000000000001");
        let mut mock = MockReadRepository::new();
        mock.expect_fetch_card_by_id()
            .with(eq(id))
            .times(1)
            .return_const(None);

        let result = mock.fetch_card_by_id(&id).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn mock_write_repository_satisfies_the_trait() {
        let mut mock = MockWriteRepository::new();
        mock.expect_delete_orphaned_images()
            .times(1)
            .return_const(Vec::new());

        let result = mock.delete_orphaned_images(&[]).await;

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn mock_spoiler_queue_satisfies_the_trait() {
        let mut mock = MockSpoilerQueue::new();
        mock.expect_ack().times(1).return_const(());

        mock.ack(1, 2).await;
    }
}
