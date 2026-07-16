pub use cards_sdk::ReadRepository as CardStore;

#[cfg(test)]
pub use cards_sdk::MockReadRepository as MockCardStore;

/// Test-only combined double: `App`'s `CS` bound is `CardStore + SpoilerQueue`
/// (one repository trait, kept as a single generic parameter, mirroring how
/// the *production* `cards_sdk::Postgres` value satisfies both traits at
/// once) — but `mockall::automock` mints one mock struct per trait, so
/// existing tests that only exercise `ReadRepository` methods need a
/// zero-behavior `SpoilerQueue` delegate bolted on to keep typechecking.
#[cfg(test)]
pub struct TestCardStore {
    pub read: MockCardStore,
    pub spoiler: cards_sdk::MockSpoilerQueue,
}

#[cfg(test)]
impl TestCardStore {
    pub fn new(read: MockCardStore) -> Self {
        Self {
            read,
            spoiler: cards_sdk::MockSpoilerQueue::new(),
        }
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl cards_sdk::ReadRepository for TestCardStore {
    async fn search(&self, normalised_name: &str) -> Option<Vec<contracts::card::Card>> {
        self.read.search(normalised_name).await
    }
    async fn search_artist(
        &self,
        artist: &str,
        normalised_name: &str,
    ) -> Option<Vec<contracts::card::Card>> {
        self.read.search_artist(artist, normalised_name).await
    }
    async fn search_set(
        &self,
        set_name: &str,
        normalised_name: &str,
    ) -> Option<Vec<contracts::card::Card>> {
        self.read.search_set(set_name, normalised_name).await
    }
    async fn search_for_set_name(&self, normalised_name: &str) -> Option<Vec<String>> {
        self.read.search_for_set_name(normalised_name).await
    }
    async fn set_name_from_abbreviation(&self, abbreviation: &str) -> Option<String> {
        self.read.set_name_from_abbreviation(abbreviation).await
    }
    async fn random_card(&self) -> Option<contracts::card::Card> {
        self.read.random_card().await
    }
    async fn random_card_from_set(&self, set_name: &str) -> Option<contracts::card::Card> {
        self.read.random_card_from_set(set_name).await
    }
    async fn all_prints(
        &self,
        oracle_id: &uuid::Uuid,
    ) -> Option<Vec<contracts::card_set::CardSet>> {
        self.read.all_prints(oracle_id).await
    }
    async fn fetch_card_by_id(&self, id: &uuid::Uuid) -> Option<contracts::card::Card> {
        self.read.fetch_card_by_id(id).await
    }
    async fn similar_cards(
        &self,
        card: &contracts::card::Card,
    ) -> Option<Vec<contracts::card::Card>> {
        self.read.similar_cards(card).await
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl cards_sdk::SpoilerQueue for TestCardStore {
    async fn subscriptions_with_pending(&self) -> Vec<cards_sdk::Subscription> {
        self.spoiler.subscriptions_with_pending().await
    }
    async fn subscription_exists(
        &self,
        guild_id: cards_sdk::GuildId,
        channel_id: cards_sdk::ChannelId,
    ) -> bool {
        self.spoiler.subscription_exists(guild_id, channel_id).await
    }
    async fn pending_cards(
        &self,
        guild_id: cards_sdk::GuildId,
        channel_id: cards_sdk::ChannelId,
        limit: i64,
    ) -> Vec<cards_sdk::PendingCard> {
        self.spoiler
            .pending_cards(guild_id, channel_id, limit)
            .await
    }
    async fn ack(
        &self,
        guild_id: cards_sdk::GuildId,
        channel_id: cards_sdk::ChannelId,
        up_to_queue_id: i64,
    ) {
        self.spoiler.ack(guild_id, channel_id, up_to_queue_id).await;
    }
    async fn create_subscription(
        &self,
        guild_id: cards_sdk::GuildId,
        channel_id: cards_sdk::ChannelId,
        sub_id: cards_sdk::SubscriptionId,
        token: &str,
    ) {
        self.spoiler
            .create_subscription(guild_id, channel_id, sub_id, token)
            .await;
    }
    async fn delete_subscription(
        &self,
        guild_id: cards_sdk::GuildId,
        channel_id: cards_sdk::ChannelId,
    ) -> Option<cards_sdk::SubscriptionId> {
        self.spoiler.delete_subscription(guild_id, channel_id).await
    }
    async fn record_failure(
        &self,
        guild_id: cards_sdk::GuildId,
        channel_id: cards_sdk::ChannelId,
    ) -> i64 {
        self.spoiler.record_failure(guild_id, channel_id).await
    }
    async fn prune_queue(&self) {
        self.spoiler.prune_queue().await;
    }
}
