use std::collections::HashMap;
use std::sync::Arc;
use log::{info, log};

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

    pub async fn card_response(
        &'a self,
        card: &Option<FoundCard<'a>>,
        msg: &Message,
        ctx: &Context,
    ) {
        match card {
            None => utils::send("Failed to find card :(", &msg, &ctx).await,
            Some(card) => {
                utils::send_image(
                    &card.image,
                    &format!("{}.png", card.query.name),
                    None,
                    &msg,
                    &ctx,
                )
                    .await;

                if let Some(image) = &card.back_image {
                    utils::send_image(
                        &image,
                        &format!("{}.png", card.query.name),
                        None,
                        &msg,
                        &ctx,
                    )
                        .await;
                }
                self.add_to_local_stores(&card).await;

                if card.score > 3 {
                    log::info!("Score is high searching scryfall for potential better match");
                    if let Some(better_card) =
                        self.mtg.find_possible_better_match(&card).await
                    {
                        log::info!("Better match found from scryfall");
                        utils::send_image(
                            &better_card.image,
                            &format!("{}.png", better_card.query.raw_name),
                            Some("I found a better match on further searches: "),
                            &msg,
                            &ctx,
                        )
                            .await;

                        if let Some(image) = &better_card.back_image {
                            utils::send_image(
                                &image,
                                &format!("{}.png", better_card.query.name),
                                None,
                                &msg,
                                &ctx,
                            )
                                .await;
                        }

                        self.add_to_local_stores(&better_card).await;
                    }
                };
            }
        }
    }
}


pub struct CardInfo {
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

impl CardInfo {
    fn new_back(card: &Scryfall, front: &CardInfo, card_id: String, other_side: &String) -> Option<Self> {
        let face = card.card_faces.as_ref()?.get(1)?;

        Some(Self {
            card_id,
            image_id: Uuid::new_v4(),
            rules_id: Uuid::new_v4(),
            legalities_id: front.legalities_id,
            name: utils::normalise(&face.name),
            flavour_text: face.flavor_text.to_owned(),
            set_id: front.set_id.clone(),
            set_name: front.set_name.clone(),
            set_code: front.set_code.clone(),
            artist: front.artist.clone(),
            legalities: front.legalities.clone(),
            colour_identity: front.colour_identity.clone(),
            mana_cost: face.mana_cost.to_owned(),
            cmc: card.cmc,
            power: face.power.to_owned(),
            toughness: face.toughness.to_owned(),
            loyalty: face.loyalty.to_owned(),
            defence: face.defence.to_owned(),
            type_line: face.type_line.to_owned(),
            oracle_text: face.oracle_text.to_owned(),
            keywords: face.keywords.to_owned(),
            other_side: Some(other_side.clone()),
        })
    }

    fn new_card(card: &Scryfall, other_side: Option<String>) -> Self {
        let name = if let Some(sides) = &card.card_faces {
            if let Some(front) = sides.get(0) {
                utils::normalise(&front.name)
            } else {
                utils::normalise(&card.name)
            }
        } else {
            utils::normalise(&card.name)
        };

        Self {
            card_id: card.id.to_owned(),
            image_id: Uuid::new_v4(),
            rules_id: Uuid::new_v4(),
            legalities_id: Uuid::new_v4(),
            name,
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
            other_side,
        }
    }
}

struct QueryParams<'a> {
    name: String,
    raw_name: &'a str,
    set_code: Option<&'a str>,
    set_name: Option<&'a str>,
    artist: Option<&'a str>,
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
            }
            None => (None, None),
        };

        let artist = match capture.get(7) {
            Some(artist) => Some(artist.as_str()),
            None => None,
        };

        Some(Self {
            name,
            raw_name,
            artist,
            set_code,
            set_name,
        })
    }
}

pub struct FoundCard<'a> {
    pub query: Arc<QueryParams<'a>>,
    pub front: Option<CardInfo>,
    pub back: Option<CardInfo>,
    pub image: Vec<u8>,
    pub back_image: Option<Vec<u8>>,
    pub score: usize,
}

impl<'a> FoundCard<'a> {
    fn new_2_faced_card(
        query: Arc<QueryParams<'a>>,
        card: &Scryfall,
        images: Vec<Option<Vec<u8>>>,
    ) -> Option<Self> {
        let back_id = Uuid::new_v4().to_string();

        let front = CardInfo::new_card(&card, Some(back_id.clone()));
        let back = CardInfo::new_back(&card, &front, back_id, &front.card_id);

        Some(Self {
            query: Arc::clone(&query),
            image: images.get(0)?.to_owned()?,
            back_image: images.get(1)?.to_owned(),
            front: Some(front),
            back,
            score: 0,
        })
    }

    fn new_card(query: Arc<QueryParams<'a>>, card: &Scryfall, image: Vec<u8>) -> Self {
        Self {
            query: Arc::clone(&query),
            image,
            front: Some(CardInfo::new_card(&card, None)),
            back_image: None,
            back: None,
            score: 0,
        }
    }

    fn existing_card(query: Arc<QueryParams<'a>>, images: Vec<Vec<u8>>, score: usize) -> Option<Self> {
        let font_image = images.get(0)?.to_owned();
        let back_image = match images.get(1) {
            Some(image) => Some(image.to_owned()),
            None => None
        };

        Some(Self {
            query: Arc::clone(&query),
            image: font_image,
            back_image,
            score,
            front: None,
            back: None,
        })
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
