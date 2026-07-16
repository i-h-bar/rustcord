use crate::emoji::add_emoji;
use contracts::card::Card;

pub async fn create_title(card: &Card) -> String {
    let mana_cost = add_emoji(card.mana_cost()).await;

    format!("{}        {}", card.name(), mana_cost)
}
