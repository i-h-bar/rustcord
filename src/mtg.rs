use lazy_static::lazy_static;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use serenity::all::Message;
use serenity::prelude::*;

const SCRYFALL: &str = "https://api.scryfall.com/cards/named?fuzzy=";

lazy_static! {
    static ref CARD_REGEX: Regex = Regex::new(r"\[\[(.*?)]]").expect("Invalid regex");
    static ref CLIENT: Client = Client::builder()
        .default_headers(HeaderMap::from_iter([(
            USER_AGENT,
            HeaderValue::from_static("Rust Discord Bot")
        )]))
        .build()
        .expect("Failed HTTP Client build");
}

pub async fn find_cards(msg: &Message, ctx: &Context) {
    for capture in CARD_REGEX.captures_iter(&msg.content) {
        let Some(name) = capture.get(1) else {
            continue;
        };
        let Ok(response) = CLIENT
            .get(format!("{}{}", SCRYFALL, name.as_str()))
            .send()
            .await
            .expect("Failed request")
            .text()
            .await
        else {
            continue;
        };

        println!("{response}");
    }
}
