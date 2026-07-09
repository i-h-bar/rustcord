use crate::ingest::{CardInfo, UpsertResult};
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
}