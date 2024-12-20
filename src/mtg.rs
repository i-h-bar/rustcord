use std::collections::HashMap;
use std::sync::Arc;

use crate::db::PSQL;
use crate::mtg::db::FuzzyFound;
use crate::{utils, Handler};
use db::QueryParams;
use serde::Deserialize;
use serenity::all::{Context, Message};
use uuid::Uuid;

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
            }
        }
    }
}

pub struct CardInfo {
    card_id: String,
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
    fn new_back(
        card: &ScryfallCard,
        front: &CardInfo,
        card_id: String,
        other_side: &String,
    ) -> Option<Self> {
        let face = card.card_faces.as_ref()?.get(1)?;

        Some(Self {
            card_id,
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

    fn new_card(card: &ScryfallCard, other_side: Option<String>) -> Self {
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

pub struct FoundCard<'a> {
    pub query: Arc<QueryParams<'a>>,
    pub front: Option<CardInfo>,
    pub back: Option<CardInfo>,
    pub image: Vec<u8>,
    pub back_image: Option<Vec<u8>>,
}

impl<'a> FoundCard<'a> {
    fn new_2_faced_card(
        query: Arc<QueryParams<'a>>,
        card: &ScryfallCard,
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
        })
    }

    fn new_card(query: Arc<QueryParams<'a>>, card: &ScryfallCard, image: Vec<u8>) -> Self {
        Self {
            query: Arc::clone(&query),
            image,
            front: Some(CardInfo::new_card(&card, None)),
            back_image: None,
            back: None,
        }
    }

    fn existing_card(
        query: Arc<QueryParams<'a>>,
        front: FuzzyFound,
        back: Option<FuzzyFound>,
    ) -> Option<Self> {
        let font_image = front.png.to_owned();
        let back_image = match back {
            Some(found) => Some(found.png.to_owned()),
            None => None,
        };

        Some(Self {
            query: Arc::clone(&query),
            image: font_image,
            back_image,
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
    #[allow(dead_code)]
    art_crop: String,
    #[allow(dead_code)]
    border_crop: String,
    #[allow(dead_code)]
    large: String,
    #[allow(dead_code)]
    normal: String,
    pub png: String,
    #[allow(dead_code)]
    small: String,
}

#[derive(Deserialize)]
struct CardFace {
    #[allow(dead_code)]
    object: String,
    name: String,
    mana_cost: Option<String>,
    type_line: String,
    oracle_text: Option<String>,
    #[allow(dead_code)]
    colors: Vec<String>,
    defence: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
    loyalty: Option<String>,
    #[allow(dead_code)]
    artist: String,
    #[allow(dead_code)]
    artist_id: String,
    #[allow(dead_code)]
    illustration_id: String,
    flavor_text: Option<String>,
    keywords: Option<Vec<String>>,
    image_uris: ImageURIs,
}

#[derive(Deserialize)]
pub struct ScryfallList {
    object: String,
    total_cards: u16,
    has_more: bool,
    data: Vec<ScryfallCard>
}

#[derive(Deserialize)]
pub struct ScryfallCard {
    artist: String,
    #[allow(dead_code)]
    artist_ids: Vec<String>,
    #[allow(dead_code)]
    booster: bool,
    #[allow(dead_code)]
    border_color: String,
    #[allow(dead_code)]
    card_back_id: Option<String>,
    card_faces: Option<Vec<CardFace>>,
    #[allow(dead_code)]
    cardmarket_id: Option<u32>,
    cmc: f32,
    #[allow(dead_code)]
    collector_number: String,
    color_identity: Vec<String>,
    #[allow(dead_code)]
    colors: Option<Vec<String>>,
    loyalty: Option<String>,
    defence: Option<String>,
    #[allow(dead_code)]
    digital: bool,
    #[allow(dead_code)]
    edhrec_rank: Option<u32>,
    #[allow(dead_code)]
    finishes: Vec<String>,
    flavor_text: Option<String>,
    #[allow(dead_code)]
    foil: bool,
    #[allow(dead_code)]
    frame: String,
    #[allow(dead_code)]
    full_art: bool,
    #[allow(dead_code)]
    games: Vec<String>,
    #[allow(dead_code)]
    highres_image: bool,
    id: String,
    #[allow(dead_code)]
    illustration_id: Option<String>,
    #[allow(dead_code)]
    image_status: String,
    image_uris: Option<ImageURIs>,
    keywords: Option<Vec<String>>,
    #[allow(dead_code)]
    lang: String,
    #[allow(dead_code)]
    layout: String,
    legalities: Legalities,
    mana_cost: Option<String>,
    #[allow(dead_code)]
    mtgo_foil_id: Option<u32>,
    #[allow(dead_code)]
    mtgo_id: Option<u32>,
    #[allow(dead_code)]
    multiverse_ids: Vec<u32>,
    name: String,
    #[allow(dead_code)]
    nonfoil: bool,
    #[allow(dead_code)]
    object: String,
    #[allow(dead_code)]
    oracle_id: String,
    oracle_text: Option<String>,
    #[allow(dead_code)]
    oversized: bool,
    #[allow(dead_code)]
    penny_rank: Option<u32>,
    power: Option<String>,
    #[allow(dead_code)]
    prices: HashMap<String, Option<String>>,
    #[allow(dead_code)]
    prints_search_uri: String,
    #[allow(dead_code)]
    promo: bool,
    #[allow(dead_code)]
    purchase_uris: Option<HashMap<String, String>>,
    #[allow(dead_code)]
    rarity: String,
    #[allow(dead_code)]
    related_uris: HashMap<String, String>,
    #[allow(dead_code)]
    released_at: String,
    #[allow(dead_code)]
    reprint: bool,
    #[allow(dead_code)]
    reserved: bool,
    #[allow(dead_code)]
    rulings_uri: String,
    #[allow(dead_code)]
    scryfall_set_uri: String,
    #[allow(dead_code)]
    scryfall_uri: String,
    set: String,
    set_id: String,
    set_name: String,
    #[allow(dead_code)]
    set_search_uri: String,
    #[allow(dead_code)]
    set_type: String,
    #[allow(dead_code)]
    set_uri: String,
    #[allow(dead_code)]
    story_spotlight: bool,
    #[allow(dead_code)]
    tcgplayer_id: Option<u32>,
    #[allow(dead_code)]
    textless: bool,
    toughness: Option<String>,
    type_line: String,
    #[allow(dead_code)]
    uri: String,
    #[allow(dead_code)]
    variation: bool,
    #[allow(dead_code)]
    watermark: Option<String>
}
