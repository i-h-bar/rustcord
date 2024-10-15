use std::collections::HashSet;
use std::env;
use std::time::Duration;

use log;
use regex::Regex;
use reqwest::{Error, Response};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serenity::all::{Cache, Message};
use serenity::futures::future::join_all;
use serenity::prelude::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, Pool, Postgres, Row};
use tokio::sync::Mutex;
use tokio::time::Instant;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;
use crate::mtg::{ImageURIs, Scryfall};

use crate::utils;
use crate::utils::fuzzy;

const SCRYFALL: &str = "https://api.scryfall.com/cards/named?fuzzy=";
const LEGALITIES_INSERT: &str = r#"
INSERT INTO legalities
(id, alchemy, brawl, commander, duel, explorer, future, gladiator, historic, legacy, modern,
oathbreaker, oldschool, pauper, paupercommander, penny, pioneer, predh, premodern, standard,
standardbrawl, timeless, vintage)
values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
$15, $16, $17, $18, $19, $20, $21, $22, $23) ON CONFLICT DO NOTHING"#;
const RULES_INSERT: &str = r#"
INSERT INTO rules
(id, colour_identity, cmc, power, toughness, type_line, oracle_text, keywords, loyalty, defence, mana_cost, legalities_id)
values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) ON CONFLICT DO NOTHING
"#;
const IMAGE_INSERT: &str = r#"INSERT INTO images (id, png) values ($1, $2) ON CONFLICT DO NOTHING"#;
const SET_INSERT: &str =
    r#"INSERT INTO sets (id, name, code) values (uuid($1), $2, $3) ON CONFLICT DO NOTHING"#;
const CARD_INSERT: &str = r#"
INSERT INTO cards (id, name, flavour_text, set_id, image_id, artist, rules_id)
values (uuid($1), $2, $3, uuid($4), uuid($5), $6, $7) ON CONFLICT DO NOTHING"#;

const EXACT_MATCH: &str = r#"
select png from cardsjoin images on cards.image_id = images.id where cards.name = $1
"#;


pub struct FoundCard<'a> {
    pub name: &'a str,
    pub new_card_info: Option<Scryfall>,
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
            if let Some((matched, score)) = fuzzy::best_match_lev(&normalised_name, &*cache) {
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
        let image = self.search_scryfall_image(&card, &queried_name).await?;

        log::info!(
            "Found '{}' from scryfall in {:.2?}",
            card.name,
            start.elapsed()
        );

        Some(FoundCard {
            name: queried_name,
            image,
            new_card_info: Some(card),
        })
    }

    async fn search_single_faced_image(
        &self, card: &Scryfall, image_uris: &ImageURIs
    ) -> Option<Vec<u8>> {
        let Ok(image) = {
            match self
                .http_client
                .get(&image_uris.png)
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

    async fn search_dual_faced_image(&self, card: &Scryfall, queried_name: &str) -> Option<Vec<u8>> {
        let lev_0 = fuzzy::lev(&queried_name, &card.card_faces.as_ref()?.get(0)?.name);
        let lev_1 = fuzzy::lev(&queried_name, &card.card_faces.as_ref()?.get(1)?.name);

        if lev_0 < lev_1 {
            self.search_single_faced_image(&card, &card.card_faces.as_ref()?.get(0)?.image_uris).await
        } else {
            self.search_single_faced_image(&card, &card.card_faces.as_ref()?.get(1)?.image_uris).await
        }
    }

    async fn search_scryfall_image(&self, card: &Scryfall, queried_name: &str) -> Option<Vec<u8>> {
        match &card.image_uris {
            Some(image_uris) => { self.search_single_faced_image(&card, &image_uris).await }
            None => { self.search_dual_faced_image(&card, &queried_name).await }
        }
    }

    async fn search_scryfall_card_data(&self, queried_name: &str) -> Option<Scryfall> {
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
            match response.json::<Scryfall>().await {
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

    pub async fn add_to_postgres(&self, card: &Scryfall, image: &Vec<u8>) {
        let legalities_id = Uuid::new_v4();
        if let Err(why) = sqlx::query(LEGALITIES_INSERT)
            .bind(&legalities_id)
            .bind(&card.legalities.alchemy)
            .bind(&card.legalities.brawl)
            .bind(&card.legalities.commander)
            .bind(&card.legalities.duel)
            .bind(&card.legalities.explorer)
            .bind(&card.legalities.future)
            .bind(&card.legalities.gladiator)
            .bind(&card.legalities.historic)
            .bind(&card.legalities.legacy)
            .bind(&card.legalities.modern)
            .bind(&card.legalities.oathbreaker)
            .bind(&card.legalities.oldschool)
            .bind(&card.legalities.pauper)
            .bind(&card.legalities.paupercommander)
            .bind(&card.legalities.penny)
            .bind(&card.legalities.pioneer)
            .bind(&card.legalities.predh)
            .bind(&card.legalities.premodern)
            .bind(&card.legalities.standard)
            .bind(&card.legalities.standardbrawl)
            .bind(&card.legalities.timeless)
            .bind(&card.legalities.vintage)
            .execute(&self.pg_pool)
            .await {
            log::warn!("Failed legalities insert - {why}")
        }

        let rules_id = Uuid::new_v4();
        if let Err(why) = sqlx::query(RULES_INSERT)
            .bind(&rules_id)
            .bind(&card.color_identity)
            .bind(&card.cmc)
            .bind(&card.power)
            .bind(&card.toughness)
            .bind(&card.type_line)
            .bind(&card.oracle_text)
            .bind(&card.keywords)
            .bind(&card.loyalty)
            .bind(&card.defence)
            .bind(&card.mana_cost)
            .bind(&legalities_id)
            .execute(&self.pg_pool)
            .await {
            log::warn!("Failed legalities insert - {why}")
        }


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
            .bind(&self.normalise_card_name(&card.name))
            .bind(&card.flavor_text)
            .bind(&card.set_id)
            .bind(&image_id)
            .bind(&card.artist)
            .bind(&rules_id)
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

    pub async fn update_local_cache(&self, card: &Scryfall) {
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