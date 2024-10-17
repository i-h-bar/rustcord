use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;
use crate::utils;

pub mod search;
mod db;



#[derive(Debug)]
pub struct NewCardInfo {
    card_id: String,
    image_id: Uuid,
    rules_id: Uuid,
    legalities_id: Uuid,
    pub(crate) name: String,
    flavour_text: Option<String>,
    set_id: String,
    set_name: String,
    set_code: String,
    artist: String,
    legalities: Legalities,
    colour_identity: Vec<String>,
    mana_cost: Option<String>,
    cmc: f32,
    power: Option<String>,
    toughness: Option<String>,
    loyalty: Option<String>,
    defence: Option<String>,
    type_line: String,
    oracle_text: Option<String>,
    keywords: Option<Vec<String>>,
    other_side: Option<String>,
}

impl NewCardInfo {
    fn new_card(card: &Scryfall) -> Self {
        Self {
            card_id: card.id.to_owned(),
            image_id: Uuid::new_v4(),
            rules_id: Uuid::new_v4(),
            legalities_id: Uuid::new_v4(),
            name: utils::normalise(&card.name),
            flavour_text: card.flavor_text.to_owned(),
            set_id: card.set_id.to_owned(),
            set_name: card.set_name.to_owned(),
            set_code: card.set.to_owned(),
            artist: card.artist.to_owned(),
            legalities: card.legalities.to_owned(),
            colour_identity: card.color_identity.to_owned(),
            mana_cost: card.mana_cost.to_owned(),
            cmc: card.cmc,
            power: card.power.to_owned(),
            toughness: card.toughness.to_owned(),
            loyalty: card.loyalty.to_owned(),
            defence: card.defence.to_owned(),
            type_line: card.type_line.to_owned(),
            oracle_text: card.oracle_text.to_owned(),
            keywords: card.keywords.to_owned(),
            other_side: None,
        }
    }
    fn new_card_side(card: &Scryfall, side: usize, side_ids: &Vec<Uuid>) -> Option<Self> {
        let face = card.card_faces.as_ref()?.get(side)?.clone();
        let card_id = side_ids.get(side)?.to_string();
        let other_side = side_ids.get((side + 1) % 2)?.to_string();

        Some(Self {
            card_id,
            image_id: Uuid::new_v4(),
            rules_id: Uuid::new_v4(),
            legalities_id: Uuid::new_v4(),
            name: utils::normalise(&face.name),
            flavour_text: face.flavor_text,
            set_id: card.set_id.clone(),
            set_name: card.set_name.clone(),
            set_code: card.set.clone(),
            artist: face.artist,
            legalities: card.legalities.clone(),
            colour_identity: card.color_identity.clone(),
            mana_cost: face.mana_cost,
            cmc: card.cmc,
            power: face.power,
            toughness: face.toughness,
            loyalty: face.loyalty,
            defence: face.defence,
            type_line: face.type_line,
            oracle_text: face.oracle_text,
            keywords: face.keywords,
            other_side: Some(other_side),
        })
    }
}

pub struct FoundCard<'a> {
    pub name: &'a str,
    pub new_card_info: Option<NewCardInfo>,
    pub image: Vec<u8>,
}

impl<'a> FoundCard<'a> {
    fn new_2_faced_card(name: &'a str, card: Scryfall, images: Vec<Option<Vec<u8>>>) -> Vec<Self> {
        let side_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

        images
            .into_iter()
            .enumerate()
            .filter_map(|(i, image)| {
                Some(Self {
                    name,
                    image: image?,
                    new_card_info: NewCardInfo::new_card_side(&card, i, &side_ids),
                })
            })
            .collect()
    }

    fn new_card(name: &'a str, card: Scryfall, image: Vec<u8>) -> Vec<Self> {
        vec![Self {
            name,
            image,
            new_card_info: Some(NewCardInfo::new_card(&card)),
        }]
    }

    fn existing_card(name: &'a str, images: Vec<Vec<u8>>) -> Vec<Self> {
        images
            .into_iter()
            .map(|image| Self {
                name,
                image,
                new_card_info: None,
            })
            .collect()
    }
}


#[derive(Deserialize, Clone, Debug)]
struct Legalities {
    alchemy: String,
    brawl: String,
    commander: String,
    duel: String,
    explorer: String,
    future: String,
    gladiator: String,
    historic: String,
    legacy: String,
    modern: String,
    oathbreaker: String,
    oldschool: String,
    pauper: String,
    paupercommander: String,
    penny: String,
    pioneer: String,
    predh: String,
    premodern: String,
    standard: String,
    standardbrawl: String,
    timeless: String,
    vintage: String,
}

#[derive(Deserialize, Clone)]
struct ImageURIs {
    art_crop: String,
    border_crop: String,
    large: String,
    normal: String,
    pub png: String,
    small: String,
}

#[derive(Deserialize, Clone)]
struct CardFace {
    object: String,
    name: String,
    mana_cost: Option<String>,
    type_line: String,
    oracle_text: Option<String>,
    colors: Vec<String>,
    defence: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
    loyalty: Option<String>,
    artist: String,
    artist_id: String,
    illustration_id: String,
    flavor_text: Option<String>,
    keywords: Option<Vec<String>>,
    image_uris: ImageURIs,
}

#[derive(Deserialize)]
pub struct Scryfall {
    artist: String,
    artist_ids: Vec<String>,
    booster: bool,
    border_color: String,
    card_back_id: Option<String>,
    card_faces: Option<Vec<CardFace>>,
    cardmarket_id: Option<u32>,
    cmc: f32,
    collector_number: String,
    color_identity: Vec<String>,
    colors: Option<Vec<String>>,
    loyalty: Option<String>,
    defence: Option<String>,
    digital: bool,
    edhrec_rank: Option<u32>,
    finishes: Vec<String>,
    flavor_text: Option<String>,
    foil: bool,
    frame: String,
    full_art: bool,
    games: Vec<String>,
    highres_image: bool,
    id: String,
    illustration_id: Option<String>,
    image_status: String,
    image_uris: Option<ImageURIs>,
    keywords: Option<Vec<String>>,
    lang: String,
    layout: String,
    legalities: Legalities,
    mana_cost: Option<String>,
    mtgo_foil_id: Option<u32>,
    mtgo_id: Option<u32>,
    multiverse_ids: Vec<u32>,
    name: String,
    nonfoil: bool,
    object: String,
    oracle_id: String,
    oracle_text: Option<String>,
    oversized: bool,
    penny_rank: Option<u32>,
    power: Option<String>,
    prices: HashMap<String, Option<String>>,
    prints_search_uri: String,
    promo: bool,
    purchase_uris: HashMap<String, String>,
    rarity: String,
    related_uris: HashMap<String, String>,
    released_at: String,
    reprint: bool,
    reserved: bool,
    rulings_uri: String,
    scryfall_set_uri: String,
    scryfall_uri: String,
    set: String,
    set_id: String,
    set_name: String,
    set_search_uri: String,
    set_type: String,
    set_uri: String,
    story_spotlight: bool,
    tcgplayer_id: Option<u32>,
    textless: bool,
    toughness: Option<String>,
    type_line: String,
    uri: String,
    variation: bool,
    watermark: Option<String>,
}
