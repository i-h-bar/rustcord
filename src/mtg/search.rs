use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use log;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serenity::futures::future::join_all;
use tokio::time::Instant;

use crate::db::PSQL;
use crate::mtg::db::{FuzzyFound, QueryParams};
use crate::mtg::{FoundCard, ScryfallCard, ScryfallList};
use crate::mtg::images::ImageFetcher;
use crate::utils::{fuzzy, REGEX_COLLECTION};

const SCRYFALL: &str = "https://api.scryfall.com/cards/search?unique=prints&q=";

pub struct MTG {
    http_client: reqwest::Client,
    images: ImageFetcher
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

        let images = ImageFetcher::new();

        Self { http_client, images }
    }

    pub async fn parse_message(&'a self, msg: &'a str) -> Vec<Option<(FuzzyFound, Option<Vec<u8>>)>> {
        join_all(
            REGEX_COLLECTION
                .cards
                .captures_iter(&msg)
                .filter_map(|capture| Some(self.find_card(QueryParams::from(capture)?))),
        )
        .await
    }

    async fn search_distinct_cards(&self, normalised_name: &str) -> Option<FuzzyFound> {
        let potentials = PSQL::get()?.fuzzy_search_distinct(&normalised_name).await?;

        let (_, closest_match) = potentials
            .into_iter()
            .map(| card | {
                let distance = fuzzy::lev(&normalised_name, &card.front_normalised_name);
                (distance, card)
            })
            .min_by(| (x, _), (y, _) | x.cmp(y))?;

        Some(closest_match)
    }

    async fn find_card(&'a self, query: QueryParams<'a>) -> Option<(FuzzyFound, Option<Vec<u8>>)> {
        let start = Instant::now();

        let found_card = if let Some(set_code) = &query.set_code {
            todo!()
        } else if let Some(set_name) = &query.set_name {
            todo!()
        } else if let Some(artist) = &query.artist {
            todo!()
        } else {
            self.search_distinct_cards(&query.name).await?
        };

        log::info!(
            "Found match for query '{}' in {} ms",
            &query.name,
            start.elapsed().as_millis()
        );

        let image = self.images.fetch(&found_card).await;

        Some((found_card, image))
    }
}
