use std::collections::HashSet;
use std::env;
use std::time::Duration;

use log;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serenity::all::Message;
use serenity::futures::future::join_all;
use serenity::prelude::*;
use sqlx::{Executor, Pool, Postgres, Row};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::Mutex;
use tokio::time::Instant;
use uuid::Uuid;

use crate::mtg::response::CardResponse;
use crate::utils;

mod response;

const SCRYFALL: &str = "https://api.scryfall.com/cards/named?fuzzy=";
const IMAGE_INSERT: &str = r#"INSERT INTO images (id, png) values ($1, $2) ON CONFLICT DO NOTHING"#;
const SET_INSERT: &str =
    r#"INSERT INTO sets (id, name, code) values (uuid($1), $2, $3) ON CONFLICT DO NOTHING"#;
const CARD_INSERT: &str = r#"INSERT INTO cards (id, name, flavour_text, set_id, image_id, artist) values (uuid($1), $2, $3, uuid($4), uuid($5), $6) ON CONFLICT DO NOTHING"#;

const EXACT_MATCH: &str =
    r#"select png from cards join images on cards.image_id = images.id where cards.name = $1"#;

pub struct MTG {
    http_client: reqwest::Client,
    card_regex: Regex,
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

        let card_regex = Regex::new(r"\[\[(.*?)]]").expect("Invalid regex");

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
            pg_pool,
            card_cache,
        }
    }

    pub async fn find_cards(&self, msg: &Message) -> Vec<Vec<u8>> {
        let futures: Vec<_> = self
            .card_regex
            .captures_iter(&msg.content)
            .filter_map(|capture| {
                let name = capture.get(1)?;
                Some(self.find_card(name.as_str()))
            })
            .collect();

        let card_images: Vec<Vec<u8>> = Vec::with_capacity(futures.len());

        for result in join_all(futures).await {
            if let Some((name, image)) = result {
                self.card_cache.lock().await.insert(name);
                log::info!("Local cache updated");
            }
        }

        card_images
    }

    async fn find_card(&self, queried_name: &str) -> Option<(String, Vec<u8>)> {
        let start = Instant::now();

        let normalised_name = queried_name.to_lowercase();
        let contains = { self.card_cache.lock().await.contains(&normalised_name) };

        if contains {
            log::info!("Found exact match in cache for '{normalised_name}'!");
            let image = sqlx::query(EXACT_MATCH)
                .bind(&normalised_name)
                .fetch_one(&self.pg_pool)
                .await
                .ok()?
                .get("png");

            log::info!(
                "Found '{normalised_name}' locally in {:.2?}",
                start.elapsed()
            );

            Some(image)
        } else {
            let Some(card) = self.search_scryfall_card_data(&queried_name).await else {
                return None;
            };

            log::info!("Matched with - \"{}\". Now searching for image...", card.name);
            let Some(image) = self.search_scryfall_image(&card).await else {
                utils::send(
                    &format!("Couldn't find a card matching '{queried_name}'"),
                    &msg,
                    &ctx,
                )
                    .await;
                return None;
            };

            log::info!(
                "Found '{}' from scryfall in {:.2?}",
                card.name,
                start.elapsed()
            );
            utils::send_image(&image, &format!("{}.png", &card.name), &msg, &ctx).await;
            self.add_to_postgres(&card, &image).await;

            Some(card.name.to_lowercase())
        }
    }

    async fn search_scryfall_image(&self, card: &CardResponse) -> Option<Vec<u8>> {
        let Ok(image) = self
            .http_client
            .get(&card.image_uris.png)
            .send()
            .await
            .expect("failed image request")
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
        let response = self
            .http_client
            .get(format!("{}{}", SCRYFALL, queried_name.replace(" ", "+")))
            .send()
            .await
            .expect("Failed request");

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
                "None 200 response from scryfall - {}",
                response.status().as_str()
            );
            None
        }
    }

    async fn add_to_postgres(&self, card: &CardResponse, image: &Vec<u8>) {
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
            .bind(&card.set)
            .execute(&self.pg_pool)
            .await
        {
            log::warn!("Failed set insert - {why}")
        };

        if let Err(why) = sqlx::query(CARD_INSERT)
            .bind(&card.id)
            .bind(&card.name.to_lowercase())
            .bind(&card.flavor_text)
            .bind(&card.set_id)
            .bind(&image_id)
            .bind(&card.artist)
            .execute(&self.pg_pool)
            .await
        {
            log::warn!("Failed card insert - {why}")
        };
    }
}
