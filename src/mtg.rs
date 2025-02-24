use std::sync::Arc;
use crate::mtg::db::FuzzyFound;
use crate::{utils, Handler};
use db::QueryParams;
use serde::Deserialize;
use serenity::all::{Context, Message};

mod db;
pub mod search;
mod images;

impl<'a> Handler {
    pub async fn card_response(
        &'a self,
        card: &Option<(FuzzyFound, Option<Vec<u8>>)>,
        msg: &Message,
        ctx: &Context,
    ) {
        match card {
            None => utils::send("Failed to find card :(", &msg, &ctx).await,
            Some((card, image)) => {
                if let Some(image) = image {
                    utils::send_image(
                        &image,
                        &format!("{}.png", &card.front_name),
                        None,
                        &msg,
                        &ctx,
                    )
                        .await;
                } else {
                    utils::send("Failed to find card :(", &msg, &ctx).await;
                }
            }
        }
    }
}

pub struct FoundCard<'a> {
    pub query: Arc<QueryParams<'a>>,
    pub card: Option<FuzzyFound>,
    pub image: Vec<u8>,
    pub back_image: Option<Vec<u8>>,
    pub scryfall_list: Option<ScryfallList>,
}

impl<'a> FoundCard<'a> {
    fn existing_card(query: Arc<QueryParams<'a>>, card: FuzzyFound) -> Option<Self> {
        todo!()
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
    card_faces: Option<Vec<Arc<CardFace>>>,
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
