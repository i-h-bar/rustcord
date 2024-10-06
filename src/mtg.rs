use std::collections::HashMap;
use std::time::Duration;

use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serenity::all::{CreateAttachment, Message};
use serenity::builder::CreateMessage;
use serenity::prelude::*;

const SCRYFALL: &str = "https://api.scryfall.com/cards/named?fuzzy=";

lazy_static! {
    static ref CARD_REGEX: Regex = Regex::new(r"\[\[(.*?)]]").expect("Invalid regex");
    static ref CLIENT: Client = Client::builder()
        .default_headers(HeaderMap::from_iter([(
            USER_AGENT,
            HeaderValue::from_static("Rust Discord Bot")
        )]))
        .timeout(Duration::new(30, 0))
        .build()
        .expect("Failed HTTP Client build");
}

#[derive(Deserialize, Debug)]
struct LegalitiesResponse {
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

#[derive(Deserialize, Debug)]
struct ImageURIResponse {
    art_crop: String,
    border_crop: String,
    large: String,
    normal: String,
    png: String,
    small: String,
}

#[derive(Deserialize, Debug)]
struct CardResponse {
    artist: String,
    artist_ids: Vec<String>,
    booster: bool,
    border_color: String,
    card_back_id: String,
    cardmarket_id: Option<u32>,
    cmc: f32,
    collector_number: String,
    color_identity: Vec<String>,
    colors: Vec<String>,
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
    illustration_id: String,
    image_status: String,
    image_uris: ImageURIResponse,
    keywords: Vec<String>,
    lang: String,
    layout: String,
    legalities: LegalitiesResponse,
    mana_cost: String,
    mtgo_foil_id: Option<u32>,
    mtgo_id: Option<u32>,
    multiverse_ids: Vec<u32>,
    name: String,
    nonfoil: bool,
    object: String,
    oracle_id: String,
    oracle_text: String,
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

pub async fn find_cards(msg: &Message, ctx: &Context) {
    for capture in CARD_REGEX.captures_iter(&msg.content) {
        let Some(name) = capture.get(1) else {
            continue;
        };
        println!("Searching scryfall for \"{}\"", name.as_str());
        let response = CLIENT
            .get(format!("{}{}", SCRYFALL, name.as_str().replace(" ", "+")))
            .send()
            .await
            .expect("Failed request");

        let card = if response.status().is_success() {
            match response.json::<CardResponse>().await {
                Ok(response) => response,
                Err(why)=> {
                    println!("Error getting card from scryfall - {why:?}");
                    // let response = CLIENT
                    //     .get(format!("{}{}", SCRYFALL, name.as_str().replace(" ", "+")))
                    //     .send()
                    //     .await
                    //     .expect("Failed request");
                    // let text = response.text().await.unwrap();
                    // println!("{text}");
                    continue;
                }
            }
        } else {continue};

        println!("Matched with - \"{}\". Now searching for image...", card.name);

        let Ok(image) = CLIENT.get(card.image_uris.png).send().await.expect("failed request").bytes().await else { continue; };
        println!("Image found for - \"{}\".", card.name);
        let message = CreateMessage::new();
        let attachment = CreateAttachment::bytes(image, format!("{}.png", card.name));
        let message = message.add_file(attachment);
        msg.channel_id.send_message(&ctx.http, message).await.unwrap();
    }
}
