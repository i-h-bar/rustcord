use serde::{Deserialize, Serialize};
use time::Date;
use time::serde::format_description;
use uuid::Uuid;

format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Serialize, Deserialize)]
pub struct ImageUris {
    pub png: Option<String>,
    pub art_crop: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Legalities {
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
}

#[derive(Serialize, Deserialize)]
pub struct Prices {
    pub usd: Option<String>,
    pub usd_foil: Option<String>,
    pub usd_etched: Option<String>,
    pub eur: Option<String>,
    pub eur_foil: Option<String>,
    pub tix: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CardFace {
    pub name: String,
    pub oracle_id: Option<Uuid>,
    pub mana_cost: Option<String>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub colors: Option<Vec<String>>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub defense: Option<String>,
    pub flavor_text: Option<String>,
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<Uuid>>,
    pub illustration_id: Option<Uuid>,
    pub image_uris: Option<ImageUris>,
    pub produced_mana: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct ScryfallCard {
    pub id: Uuid,
    pub oracle_id: Option<Uuid>,
    pub name: String,
    #[serde(with = "date_format")]
    pub released_at: Date,
    pub scryfall_uri: String,
    pub flavor_text: Option<String>,
    pub reserved: bool,
    pub rarity: String,
    pub set_id: Uuid,
    #[serde(rename = "set")]
    pub set_abbreviation: String,
    pub set_name: String,
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<Uuid>>,
    pub illustration_id: Option<Uuid>,
    pub image_uris: Option<ImageUris>,
    pub legalities: Legalities,
    pub game_changer: Option<bool>,
    pub color_identity: Vec<String>,
    pub mana_cost: Option<String>,
    pub cmc: Option<f64>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub defense: Option<String>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub colors: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub produced_mana: Option<Vec<String>>,
    pub rulings_uri: Option<String>,
    pub prices: Prices,
    pub card_faces: Option<Vec<CardFace>>,
}