use std::collections::HashMap;
use regex::Captures;

use serde::Deserialize;
use serenity::all::{Context, Message};
use uuid::Uuid;

use crate::{Handler, utils};
use crate::db::PSQL;

mod db;
pub mod search;


impl<'a> Handler {
    async fn add_to_local_stores(&'a self, card_face: &FoundCard<'a>) {
        if let Some(pool) = PSQL::get() {
            pool.add_card(&card_face).await;
        }
        self.mtg.update_local_cache(&card_face).await;
    }

    pub async fn card_response(&'a self, card: &Option<Vec<FoundCard<'a>>>, msg: &Message, ctx: &Context) {
        match card {
            None => utils::send("Failed to find card :(", &msg, &ctx).await,
            Some(card) => {
                for card_face in card {
                    utils::send_image(
                        &card_face.image,
                        &format!("{}.png", card_face.name),
                        None,
                        &msg,
                        &ctx,
                    )
                        .await;

                    self.add_to_local_stores(&card_face).await;
                }

                if let Some(card_face) = card.get(0) {
                    if card_face.score > 3 {
                        log::info!("Score is high searching scryfall for potential better match");
                        if let Some(better_card) =
                            self.mtg.find_possible_better_match(&card_face).await
                        {
                            for better_face in better_card.iter() {
                                log::info!("Better match found from scryfall");
                                utils::send_image(
                                    &better_face.image,
                                    &format!("{}.png", better_face.raw_name),
                                    Some("I found a better match on further searches: "),
                                    &msg,
                                    &ctx,
                                )
                                    .await;

                                self.add_to_local_stores(&better_face).await;
                            }
                        };
                    }
                }
            }
        }
    }
}


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

struct QueryParams<'a> {
    name: String,
    raw_name: &'a str,
    set_code: Option<&'a str>,
    set_name: Option<&'a str>,
    artist: Option<&'a str>
}


impl<'a> QueryParams<'a> {
    fn from(capture: Captures<'a>) -> Option<Self> {
        let raw_name = capture.get(1)?.as_str();
        let name = utils::normalise(&raw_name);
        let (set_code, set_name) = match capture.get(4) {
            Some(set) => {
                let set = set.as_str();
                if set.chars().count() == 3 {
                    (Some(set), None)
                } else {
                    (None, Some(set))
                }
            },
            None => (None, None)
        };

        let artist = match capture.get(7) {
            Some(artist) => Some(artist.as_str()),
            None => None
        };

        Some( Self { name, raw_name, artist, set_code, set_name } )
    }
}

pub struct FoundCard<'a> {
    pub name: &'a str,
    pub raw_name: &'a str,
    pub new_card_info: Option<NewCardInfo>,
    pub image: Vec<u8>,
    pub score: usize,
}

impl<'a> FoundCard<'a> {
    fn new_2_faced_card(
        query: &'a QueryParams<'_>,
        card: &Scryfall,
        images: Vec<Option<Vec<u8>>>,
    ) -> Vec<Self> {
        let side_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

        images
            .into_iter()
            .enumerate()
            .filter_map(|(i, image)| {
                Some(Self {
                    name: &query.name,
                    raw_name: query.raw_name,
                    image: image?,
                    new_card_info: NewCardInfo::new_card_side(&card, i, &side_ids),
                    score: 0,
                })
            })
            .collect()
    }

    fn new_card(query: &'a QueryParams<'_>, card: &Scryfall, image: Vec<u8>) -> Vec<Self> {
        vec![Self {
            name: &query.name,
            raw_name: query.raw_name,
            image,
            new_card_info: Some(NewCardInfo::new_card(&card)),
            score: 0,
        }]
    }

    fn existing_card(query: &'a QueryParams<'_>, images: Vec<Vec<u8>>, score: usize) -> Vec<Self> {
        images
            .into_iter()
            .map(|image| Self {
                name: &query.name,
                raw_name: query.raw_name,
                image,
                new_card_info: None,
                score,
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
