pub mod postgres;

use crate::adapters::card_store::postgres::Postgres;
use crate::domain::card::Card;
use async_trait::async_trait;

#[async_trait]
pub trait CardStore {
    async fn new() -> Self;
    async fn search(&self, normalised_name: &str) -> Option<Vec<Card>>;
    async fn search_artist(&self, artist: &str, normalised_name: &str) -> Option<Vec<Card>>;
    async fn search_set(&self, set_name: &str, normalised_name: &str) -> Option<Vec<Card>>;
    async fn search_for_set_name(&self, normalised_name: &str) -> Option<Vec<String>>;
    async fn set_name_from_abbreviation(&self, abbreviation: &str) -> Option<String>;
    async fn random_card(&self) -> Option<Card>;
    async fn random_card_from_set(&self, set_name: &str) -> Option<Card>;
}

pub async fn init_card_store() -> impl CardStore {
    Postgres::new().await
}
