use async_trait::async_trait;
use contracts::card::Card;

#[async_trait]
pub trait CardSource {
    async fn get_recent_cards(&self) -> Vec<Card>;
}
