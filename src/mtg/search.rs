use std::collections::{HashMap, HashSet};
use std::env;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use log;
use rayon::iter::IntoParallelRefIterator;
use regex::Regex;
use reqwest::{Error, Response};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serenity::all::{Cache, Message};
use serenity::futures::future::join_all;
use serenity::prelude::*;
use sqlx::{Executor, Pool, Postgres, Row};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::Mutex;
use tokio::time::Instant;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::db::PSQL;
use crate::mtg::{CardFace, FoundCard, ImageURIs, Legalities, Scryfall};
use crate::mtg::NewCardInfo;
use crate::utils;
use crate::utils::{fuzzy, REGEX_COLLECTION};

const SCRYFALL: &str = "https://api.scryfall.com/cards/named?fuzzy=";

pub struct MTG {
    http_client: reqwest::Client,
    card_cache: Mutex<HashMap<String, String>>,
}

impl<'a> MTG {
    pub async fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Rust Discord Bot"));
        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::new(30, 0))
            .build()
            .expect("Failed HTTP Client build");

        let cards_map = PSQL::get()
            .expect("Could not retrieve instance of DB")
            .names_and_ids()
            .await;
        let card_cache = Mutex::new(cards_map);

        Self {
            http_client,
            card_cache,
        }
    }

    pub async fn find_cards(&'a self, msg: &'a str) -> Vec<Option<Vec<FoundCard<'a>>>> {
        let futures: Vec<_> = REGEX_COLLECTION
            .cards
            .captures_iter(&msg)
            .filter_map(|capture| Some(self.find_card(capture.get(1)?.as_str())))
            .collect();

        join_all(futures).await
    }

    async fn find_card(&'a self, queried_name: &'a str) -> Option<Vec<FoundCard<'a>>> {
        let start = Instant::now();

        let normalised_name = utils::normalise(&queried_name);
        log::info!("'{}' normalised to '{}'", queried_name, normalised_name);

        if let Some(id) = { self.card_cache.lock().await.get(&normalised_name) } {
            log::info!("Found exact match in cache for '{normalised_name}'!");
            let images = PSQL::get()?.fetch_card(&id).await?;

            log::info!(
                "Found '{normalised_name}' locally in {:.2?}",
                start.elapsed()
            );

            return Some(FoundCard::existing_card(queried_name, images, 0));
        };

        {
            if let Some(((matched, id), score)) =
                { fuzzy::best_match_lev_keys(&normalised_name, &*(self.card_cache.lock().await)) }
            {
                if score < 5 {
                    log::info!(
                        "Found a fuzzy in cache - '{}' with a score of {}",
                        matched,
                        score
                    );

                    let images = PSQL::get()?.fetch_card(&id).await?;

                    log::info!("Found '{matched}' fuzzily in {:.2?}", start.elapsed());

                    return Some(FoundCard::existing_card(queried_name, images, score));
                } else {
                    log::info!("Could not find a fuzzy match for '{}'", normalised_name);
                }
            }
        };

        let card = self
            .find_from_scryfall(&queried_name, &normalised_name)
            .await?;
        log::info!(
            "Found match for '{}' from scryfall in {:.2?}",
            queried_name,
            start.elapsed()
        );

        Some(card)
    }

    pub async fn find_possible_better_match(
        &'a self,
        cache_found: &FoundCard<'a>,
    ) -> Option<Vec<FoundCard<'a>>> {
        let normalised_name = utils::normalise(&cache_found.queried_name);
        let card_faces = self
            .find_from_scryfall(&cache_found.queried_name, &normalised_name)
            .await?;

        for face in card_faces.iter() {
            if fuzzy::lev(&normalised_name, &face.new_card_info.as_ref()?.name) < cache_found.score
            {
                return Some(card_faces);
            }
        }

        None
    }

    async fn find_from_scryfall(
        &'a self,
        queried_name: &'a str,
        normalised_name: &str,
    ) -> Option<Vec<FoundCard>> {
        let card = self.search_scryfall_card_data(&normalised_name).await?;
        match &card.card_faces {
            Some(card_faces) => {
                let face_0 = <Vec<CardFace> as AsRef<Vec<CardFace>>>::as_ref(card_faces).get(0)?;
                let face_1 = <Vec<CardFace> as AsRef<Vec<CardFace>>>::as_ref(card_faces).get(1)?;

                let images = join_all(vec![
                    self.search_single_faced_image(&card, &face_0.image_uris),
                    self.search_single_faced_image(&card, &face_1.image_uris),
                ])
                    .await;

                Some(FoundCard::new_2_faced_card(queried_name, &card, images))
            }
            None => {
                log::info!(
                    "Matched with - \"{}\". Now searching for image...",
                    card.name
                );
                let image = self
                    .search_single_faced_image(&card, card.image_uris.as_ref()?)
                    .await?;

                Some(FoundCard::new_card(queried_name, &card, image))
            }
        }
    }

    async fn search_single_faced_image(
        &self,
        card: &Scryfall,
        image_uris: &ImageURIs,
    ) -> Option<Vec<u8>> {
        let Ok(image) = {
            match self.http_client.get(&image_uris.png).send().await {
                Ok(response) => response,
                Err(why) => {
                    log::warn!("Error grabbing image for '{}' because: {}", card.name, why);
                    return None;
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

    async fn search_dual_faced_image(
        &'a self,
        card: &'a Scryfall,
        queried_name: &str,
    ) -> Option<(Vec<u8>, &'a CardFace)> {
        let lev_0 = fuzzy::lev(&queried_name, &card.card_faces.as_ref()?.get(0)?.name);
        let lev_1 = fuzzy::lev(&queried_name, &card.card_faces.as_ref()?.get(1)?.name);

        if lev_0 < lev_1 {
            Some((
                self.search_single_faced_image(
                    &card,
                    &card.card_faces.as_ref()?.get(0)?.image_uris,
                )
                    .await?,
                card.card_faces.as_ref()?.get(0)?,
            ))
        } else {
            Some((
                self.search_single_faced_image(
                    &card,
                    &card.card_faces.as_ref()?.get(1)?.image_uris,
                )
                    .await?,
                card.card_faces.as_ref()?.get(1)?,
            ))
        }
    }

    async fn search_scryfall_card_data(&self, queried_name: &str) -> Option<Scryfall> {
        log::info!("Searching scryfall for '{queried_name}'");
        let response = match self
            .http_client
            .get(format!("{}{}", SCRYFALL, queried_name.replace(" ", "+")))
            .send()
            .await
        {
            Ok(response) => response,
            Err(why) => {
                log::warn!("Error searching for '{}' because: {}", queried_name, why);
                return None;
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
            match response.status().as_u16() {
                404 => {
                    log::info!("Could not find card from scryfall with the name '{}'", queried_name)
                }
                status => {
                    log::warn!(
                        "None 200 response from scryfall: {} when searching for '{}'",
                        status,
                        queried_name
                    );
                }
            }

            None
        }
    }

    pub async fn update_local_cache(&self, card: &FoundCard<'a>) {
        if let Some(new_card) = &card.new_card_info {
            self.card_cache
                .lock()
                .await
                .insert(new_card.name.to_string(), new_card.card_id.to_string());
            log::info!("Added '{}' to local cache", new_card.name);
        }
    }
}
