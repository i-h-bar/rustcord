use regex::Captures;
use contracts::card::Card;
use crate::adapters::drivers::discord::utils::emoji::add_emoji;
use crate::domain::utils::REGEX_COLLECTION;

pub fn create_title(card: &Card) -> String {
    let mana_cost = REGEX_COLLECTION
        .symbols
        .replace_all(card.mana_cost(), |cap: &Captures| add_emoji(cap));
    
    format!("{}        {}", card.name(), mana_cost)
}