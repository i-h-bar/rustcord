use async_trait::async_trait;
use contracts::card::Card;
use contracts::set::Set;

#[async_trait]
pub trait CardSource {
    async fn get_recent_sets(&self) -> Vec<Set>;
    async fn get_cards_from_set(&self, sets: &Vec<Set>) -> Vec<Card>;
}
