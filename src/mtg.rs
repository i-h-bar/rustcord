use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use crate::db::PSQL;
use crate::mtg::db::FuzzyFound;
use crate::{utils, Handler};
use db::QueryParams;
use serde::Deserialize;
use serenity::all::{Context, Message};
use serenity::futures::future::join_all;
use uuid::Uuid;

mod db;
pub mod search;

impl<'a> Handler {
    async fn add_to_local_stores(&'a self, found_card: &FoundCard<'a>) {
        if let Some(pool) = PSQL::get() {
            if let Some(cards) = &found_card.scryfall_list {
                let found_cards = join_all(cards.data.iter().map(|card| {
                    self.mtg
                        .create_found_card(Arc::clone(&found_card.query), &card, None)
                }))
                .await;

                let mut shared_ids: HashMap<&str, (Uuid, Uuid)> = HashMap::new();
                for card_option in found_cards.iter() {
                    if let Some(card) = card_option.as_ref() {
                        let legalities_id = if let Some(front) = card.front.as_ref() {
                            if !shared_ids.contains_key(&front.name.as_ref()) {
                                if let Some((legalities_id, rules_id)) =
                                    pool.fetch_rules_legalities_id(&front.name).await
                                {
                                    shared_ids.insert(&front.name, (legalities_id, rules_id));
                                    legalities_id
                                } else {
                                    let legalities_id = Uuid::new_v4();
                                    shared_ids.insert(&front.name, (legalities_id, Uuid::new_v4()));
                                    legalities_id
                                }
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        };

                        if let Some(back) = card.back.as_ref() {
                            if !shared_ids.contains_key(&back.name.as_ref()) {
                                if let Some((legalities_id, rules_id)) =
                                    pool.fetch_rules_legalities_id(&back.name).await
                                {
                                    shared_ids.insert(&back.name, (legalities_id, rules_id));
                                } else {
                                    shared_ids.insert(&back.name, (legalities_id, Uuid::new_v4()));
                                }
                            }
                        }
                    }
                }

                join_all(
                    found_cards
                        .iter()
                        .filter_map(|card| Some(pool.add_card(card.as_ref()?, &shared_ids))),
                )
                .await;
            }
        }
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
    card_id: Arc<str>,
    legalities_id: Uuid,
    pub(crate) name: String,
    flavour_text: Arc<Option<Box<str>>>,
    set_id: Arc<str>,
    set_name: Arc<str>,
    set_code: Arc<str>,
    artist: Arc<str>,
    legalities: Arc<Legalities>,
    colour_identity: Arc<[Box<str>]>,
    mana_cost: Arc<Option<Box<str>>>,
    cmc: f32,
    power: Arc<Option<Box<str>>>,
    toughness: Arc<Option<Box<str>>>,
    loyalty: Arc<Option<Box<str>>>,
    defence: Arc<Option<Box<str>>>,
    type_line: Arc<Option<Box<str>>>,
    oracle_text: Arc<Option<Box<str>>>,
    keywords: Arc<Option<Vec<Box<str>>>>,
    other_side: Option<Arc<str>>,
}

impl CardInfo {
    fn new_back(
        card: &ScryfallCard,
        front: &CardInfo,
        card_id: Arc<str>,
        other_side: Arc<str>,
    ) -> Option<Self> {
        let face = card.card_faces.deref().as_ref()?.get(1)?;

        Some(Self {
            card_id,
            legalities_id: front.legalities_id,
            name: utils::normalise(&face.name),
            flavour_text: Arc::clone(&face.flavor_text),
            set_id: Arc::clone(&front.set_id),
            set_name: Arc::clone(&front.set_name),
            set_code: Arc::clone(&front.set_code),
            artist: Arc::clone(&front.artist),
            legalities: Arc::clone(&front.legalities),
            colour_identity: Arc::clone(&front.colour_identity),
            mana_cost: Arc::clone(&face.mana_cost),
            cmc: card.cmc.unwrap_or_else(|| 0.0),
            power: Arc::clone(&face.power),
            toughness: Arc::clone(&face.toughness),
            loyalty: Arc::clone(&face.loyalty),
            defence: Arc::clone(&face.defence),
            type_line: Arc::clone(&face.type_line),
            oracle_text: Arc::clone(&face.oracle_text),
            keywords: Arc::clone(&face.keywords),
            other_side: Some(Arc::clone(&other_side)),
        })
    }

    fn new_card(card: &ScryfallCard, other_side: Option<Arc<str>>) -> Self {
        let name = if let Some(sides) = &card.card_faces.deref() {
            if let Some(front) = sides.get(0) {
                utils::normalise(&front.name)
            } else {
                utils::normalise(&card.name)
            }
        } else {
            utils::normalise(&card.name)
        };

        Self {
            card_id: Arc::clone(&card.id),
            legalities_id: Uuid::new_v4(),
            name,
            flavour_text: Arc::clone(&card.flavor_text),
            set_id: Arc::clone(&card.set_id),
            set_name: Arc::clone(&card.set_name),
            set_code: Arc::clone(&card.set),
            artist: Arc::clone(&card.artist),
            legalities: Arc::clone(&card.legalities),
            colour_identity: Arc::clone(&card.color_identity),
            mana_cost: Arc::clone(&card.mana_cost),
            cmc: card.cmc.unwrap_or_else(|| 0.0),
            power: Arc::clone(&card.power),
            toughness: Arc::clone(&card.toughness),
            loyalty: Arc::clone(&card.loyalty),
            defence: Arc::clone(&card.defence),
            type_line: Arc::clone(&card.type_line),
            oracle_text: Arc::clone(&card.oracle_text),
            keywords: Arc::clone(&card.keywords),
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
    pub scryfall_list: Option<ScryfallList>,
}

impl<'a> FoundCard<'a> {
    fn new_2_faced_card(
        query: Arc<QueryParams<'a>>,
        card: &ScryfallCard,
        images: Vec<Vec<u8>>,
        scryfall_list: Option<ScryfallList>,
    ) -> Option<Self> {
        let back_id = Uuid::new_v4().to_string().as_str().into();

        let front = CardInfo::new_card(&card, Some(Arc::clone(&back_id)));
        let back = CardInfo::new_back(&card, &front, Arc::clone(&back_id), Arc::clone(&front.card_id));

        Some(Self {
            query: Arc::clone(&query),
            image: images.get(0)?.to_owned(),
            back_image: Some(images.get(1)?.to_owned()),
            front: Some(front),
            back,
            scryfall_list,
        })
    }

    fn new_card(
        query: Arc<QueryParams<'a>>,
        card: &ScryfallCard,
        image: Vec<u8>,
        scryfall_list: Option<ScryfallList>,
    ) -> Self {
        Self {
            query: Arc::clone(&query),
            image,
            front: Some(CardInfo::new_card(&card, None)),
            back_image: None,
            back: None,
            scryfall_list,
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
            scryfall_list: None,
        })
    }
}

#[derive(Deserialize, Clone, Debug)]
struct Legalities {
    alchemy: Box<str>,
    brawl: Box<str>,
    commander: Box<str>,
    duel: Box<str>,
    explorer: Box<str>,
    future: Box<str>,
    gladiator: Box<str>,
    historic: Box<str>,
    legacy: Box<str>,
    modern: Box<str>,
    oathbreaker: Box<str>,
    oldschool: Box<str>,
    pauper: Box<str>,
    paupercommander: Box<str>,
    penny: Box<str>,
    pioneer: Box<str>,
    predh: Box<str>,
    premodern: Box<str>,
    standard: Box<str>,
    standardbrawl: Box<str>,
    timeless: Box<str>,
    vintage: Box<str>,
}

#[derive(Deserialize, Clone)]
struct ImageURIs {
    pub png: Box<str>,
}

#[derive(Deserialize, Clone)]
struct CardFace {
    name: Arc<str>,
    mana_cost: Arc<Option<Box<str>>>,
    type_line: Arc<Option<Box<str>>>,
    oracle_text: Arc<Option<Box<str>>>,
    defence: Arc<Option<Box<str>>>,
    power: Arc<Option<Box<str>>>,
    toughness: Arc<Option<Box<str>>>,
    loyalty: Arc<Option<Box<str>>>,
    flavor_text: Arc<Option<Box<str>>>,
    keywords: Arc<Option<Vec<Box<str>>>>,
    image_uris: Arc<ImageURIs>,
}

#[derive(Deserialize, Clone)]
pub struct ScryfallList {
    data: Vec<ScryfallCard>,
}

#[derive(Deserialize, Clone)]
pub struct ScryfallCard {
    artist: Arc<str>,
    card_faces: Arc<Option<Vec<Arc<CardFace>>>>,
    cmc: Option<f32>,
    color_identity: Arc<[Box<str>]>,
    loyalty: Arc<Option<Box<str>>>,
    defence: Arc<Option<Box<str>>>,
    flavor_text: Arc<Option<Box<str>>>,
    id: Arc<str>,
    image_uris: Arc<Option<ImageURIs>>,
    keywords: Arc<Option<Vec<Box<str>>>>,
    legalities: Arc<Legalities>,
    mana_cost: Arc<Option<Box<str>>>,
    name: Arc<str>,
    oracle_text: Arc<Option<Box<str>>>,
    power: Arc<Option<Box<str>>>,
    set: Arc<str>,
    set_id: Arc<str>,
    set_name: Arc<str>,
    toughness: Arc<Option<Box<str>>>,
    type_line: Arc<Option<Box<str>>>,
}
