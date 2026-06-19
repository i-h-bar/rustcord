use async_trait::async_trait;
use std::collections::HashSet;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

#[async_trait]
pub trait Storage {
    async fn get_existing_card_ids(&self, sets: Vec<Set>) -> Vec<(Set, HashSet<Uuid>)>;
    async fn upsert_cards(&self, cards: &[CardInfo]);
}

pub struct Set {
    pub id: Uuid,
    pub name: String,
    pub normalised_name: String,
    pub abbreviation: String,
}

pub struct Artist {
    pub id: Uuid,
    pub name: String,
    pub normalised_name: String,
}

pub struct Image {
    pub id: Uuid,
    pub scryfall_url: String,
}

pub struct Illustration {
    pub id: Uuid,
    pub scryfall_url: String,
}

pub struct Legality {
    pub id: Uuid,
    pub alchemy: String,
    pub brawl: String,
    pub commander: String,
    pub duel: String,
    pub future: String,
    pub gladiator: String,
    pub historic: String,
    pub legacy: String,
    pub modern: String,
    pub oathbreaker: String,
    pub oldschool: String,
    pub pauper: String,
    pub paupercommander: String,
    pub penny: String,
    pub pioneer: String,
    pub predh: String,
    pub premodern: String,
    pub standard: String,
    pub standardbrawl: String,
    pub timeless: String,
    pub vintage: String,
    pub game_changer: bool,
}

pub struct Rule {
    pub id: Uuid,
    pub colour_identity: Vec<String>,
    pub mana_cost: Option<String>,
    pub cmc: f64,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub defence: Option<String>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub colours: Vec<String>,
    pub keywords: Vec<String>,
    pub produced_mana: Option<Vec<String>>,
    pub rulings_url: Option<String>,
}

pub struct Card {
    pub id: Uuid,
    pub oracle_id: Uuid,
    pub name: String,
    pub normalised_name: String,
    pub scryfall_url: String,
    pub flavour_text: Option<String>,
    pub release_date: Date,
    pub reserved: bool,
    pub rarity: String,
    pub artist_id: Uuid,
    pub image_id: Uuid,
    pub illustration_id: Option<Uuid>,
    pub set_id: Uuid,
    pub backside_id: Option<Uuid>,
}

pub struct Price {
    pub id: Uuid,
    pub usd: Option<f64>,
    pub usd_foil: Option<f64>,
    pub usd_etched: Option<f64>,
    pub euro: Option<f64>,
    pub euro_foil: Option<f64>,
    pub tix: Option<f64>,
    pub updated_time: OffsetDateTime,
}

pub struct Combo {
    pub id: Uuid,
    pub card_id: Uuid,
    pub combo_card_id: Uuid,
}

pub struct RelatedToken {
    pub id: Uuid,
    pub card_id: Uuid,
    pub token_id: Uuid,
}

pub struct CardInfo {
    pub card: Card,
    pub artist: Artist,
    pub image: Image,
    pub illustration: Option<Illustration>,
    pub set: Set,
    pub rule: Rule,
    pub legality: Legality,
    pub price: Price,
    pub combos: Vec<Combo>,
    pub related_tokens: Vec<RelatedToken>,
}
