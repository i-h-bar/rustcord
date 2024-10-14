use std::collections::HashSet;
use std::env;
use std::time::Duration;

use log;
use regex::Regex;
use reqwest::{Error, Response};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serenity::all::Message;
use serenity::futures::future::join_all;
use serenity::prelude::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, Pool, Postgres, Row};
use tokio::sync::Mutex;
use tokio::time::Instant;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::mtg::response::CardResponse;
use crate::utils;
use crate::utils::fuzzy;

mod response;

const SCRYFALL: &str = "https://api.scryfall.com/cards/named?fuzzy=";
const IMAGE_INSERT: &str = r#"INSERT INTO images (id, png) values ($1, $2) ON CONFLICT DO NOTHING"#;
const SET_INSERT: &str =
    r#"INSERT INTO sets (id, name, code) values (uuid($1), $2, $3) ON CONFLICT DO NOTHING"#;
const CARD_INSERT: &str = r#"INSERT INTO cards (id, name, flavour_text, set_id, image_id, artist) values (uuid($1), $2, $3, uuid($4), uuid($5), $6) ON CONFLICT DO NOTHING"#;

const EXACT_MATCH: &str =
    r#"select png from cards join images on cards.image_id = images.id where cards.name = $1"#;

pub struct CardInfo {
    id: String,
    name: String,
    set_id: String,
    set_name: String,
    set_code: String,
    artist: String,
    flavour_text: Option<String>,
}

pub struct FoundCard<'a> {
    pub name: &'a str,
    pub new_card_info: Option<CardInfo>,
    pub image: Vec<u8>,
}

pub struct MTG {
    http_client: reqwest::Client,
    card_regex: Regex,
    punc_regex: Regex,
    pg_pool: Pool<Postgres>,
    card_cache: Mutex<HashSet<String>>,
}

impl MTG {
    pub async fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Rust Discord Bot"));
        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::new(30, 0))
            .build()
            .expect("Failed HTTP Client build");

        let card_regex = Regex::new(r#"\[\[(.*?)]]"#).expect("Invalid regex");
        let punc_regex = Regex::new(r#"[^\w\s]"#).expect("Invalid regex");

        let uri = env::var("PSQL_URI").expect("Postgres uri wasn't in env vars");
        let pg_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&uri)
            .await
            .expect("Failed Postgres connection");

        let cards_vec = sqlx::query("select name from cards")
            .fetch_all(&pg_pool)
            .await
            .expect("Failed to get cards names");

        let card_cache = Mutex::new(
            cards_vec
                .into_iter()
                .map(|row| row.get("name"))
                .collect::<HashSet<String>>(),
        );

        Self {
            http_client,
            card_regex,
            punc_regex,
            pg_pool,
            card_cache,
        }
    }

    pub async fn find_cards<'a>(&'a self, msg: &'a str) -> Vec<Option<FoundCard<'a>>> {
        let futures: Vec<_> = self
            .card_regex
            .captures_iter(&msg)
            .filter_map(|capture| Some(self.find_card(capture.get(1)?.as_str())))
            .collect();

        join_all(futures).await
    }

    async fn find_card<'a>(&'a self, queried_name: &'a str) -> Option<FoundCard<'a>> {
        let start = Instant::now();

        let normalised_name = self.normalise_card_name(&queried_name);
        log::info!("'{}' normalised to '{}'", queried_name, normalised_name);
        let contains = { self.card_cache.lock().await.contains(&normalised_name) };
        if contains {
            log::info!("Found exact match in cache for '{normalised_name}'!");
            let image = self.fetch_local(&normalised_name).await?;

            log::info!(
                "Found '{normalised_name}' locally in {:.2?}",
                start.elapsed()
            );

            return Some(FoundCard {
                name: queried_name,
                image,
                new_card_info: None,
            });
        };

        {
            let cache = self.card_cache.lock().await;
            if let Some((matched, score)) = fuzzy::best_match(&normalised_name, &*cache) {
                if score < 6 {
                    log::info!("Found a fuzzy in cache - '{}' with a score of {}", matched, score);
                    let image = self.fetch_local(&matched).await?;
                    log::info!("Found '{matched}' fuzzily in {:.2?}", start.elapsed());

                    return Some(FoundCard {
                        name: queried_name,
                        image,
                        new_card_info: None,
                    });
                } else {
                    log::info!("Could not find a fuzzy match for '{}'", normalised_name);
                }
            }
        };

        let card = self.search_scryfall_card_data(&normalised_name).await?;

        log::info!(
            "Matched with - \"{}\". Now searching for image...",
            card.name
        );
        let image = self.search_scryfall_image(&card).await?;

        log::info!(
            "Found '{}' from scryfall in {:.2?}",
            card.name,
            start.elapsed()
        );

        Some(FoundCard {
            name: queried_name,
            image,
            new_card_info: Some(CardInfo {
                id: card.id,
                name: card.name,
                set_name: card.set_name,
                set_code: card.set,
                set_id: card.set_id,
                artist: card.artist,
                flavour_text: card.flavor_text,
            }),
        })
    }

    async fn search_scryfall_image(&self, card: &CardResponse) -> Option<Vec<u8>> {
        let Ok(image) = {
            match self
                .http_client
                .get(&card.image_uris.png)
                .send()
                .await {
                Ok(response) => response,
                Err(why) => {
                    log::warn!("Error grabbing image for '{}' because: {}", card.name, why);
                    return None
                }
            }
        }
            .bytes()
            .await
        else {
            log::warn!("Failed to retrieve image bytes");
            return None;
        };

        log::info!("Image found for - \"{}\".", &card.name);
        Some(image.to_vec())
    }

    async fn search_scryfall_card_data(&self, queried_name: &str) -> Option<CardResponse> {
        log::info!("Searching scryfall for '{queried_name}'");
        let response = match self
            .http_client
            .get(format!("{}{}", SCRYFALL, queried_name.replace(" ", "+")))
            .send()
            .await {
            Ok(response) => response,
            Err(why) => {
                log::warn!("Error searching for '{}' because: {}", queried_name, why);
                return None
            }
        };

        if response.status().is_success() {
            match response.json::<CardResponse>().await {
                Ok(response) => Some(response),
                Err(why) => {
                    log::warn!("Error getting card from scryfall - {why:?}");
                    None
                }
            }
        } else {
            log::warn!(
                "None 200 response from scryfall: {} when searching for '{}'",
                response.status().as_str(),
                queried_name
            );
            None
        }
    }

    pub async fn add_to_postgres(&self, card: &CardInfo, image: &Vec<u8>) {
        let image_id = Uuid::new_v4();
        if let Err(why) = sqlx::query(IMAGE_INSERT)
            .bind(&image_id)
            .bind(&image)
            .execute(&self.pg_pool)
            .await
        {
            log::warn!("Failed images insert - {why}")
        };

        if let Err(why) = sqlx::query(SET_INSERT)
            .bind(&card.set_id)
            .bind(&card.set_name)
            .bind(&card.set_code)
            .execute(&self.pg_pool)
            .await
        {
            log::warn!("Failed set insert - {why}")
        };

        if let Err(why) = sqlx::query(CARD_INSERT)
            .bind(&card.id)
            .bind(&self.normalise_card_name(&card.name))
            .bind(&card.flavour_text)
            .bind(&card.set_id)
            .bind(&image_id)
            .bind(&card.artist)
            .execute(&self.pg_pool)
            .await
        {
            log::warn!("Failed card insert - {why}")
        };

        log::info!("Added {} to postgres", card.name)
    }

    async fn fetch_local(&self, matched: &str) -> Option<Vec<u8>> {
        sqlx::query(EXACT_MATCH)
            .bind(&matched)
            .fetch_one(&self.pg_pool)
            .await
            .ok()?
            .get("png")
    }

    pub async fn update_local_cache(&self, card: &CardInfo) {
        self.card_cache
            .lock()
            .await
            .insert(self.normalise_card_name(&card.name));
        log::info!("Added '{}' to local cache", card.name);
    }

    fn normalise_card_name(&self, name: &str) -> String {
        self.punc_regex.replace(&name.nfkc().collect::<String>(), "").to_lowercase()
    }
}
