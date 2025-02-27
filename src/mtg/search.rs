use std::time::Duration;

use log;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serenity::all::Embed;
use serenity::builder::CreateEmbed;
use serenity::futures::future::join_all;
use tokio::time::Instant;

use crate::db::PSQL;
use crate::mtg::db::{FuzzyFound, QueryParams};
use crate::mtg::images::ImageFetcher;
use crate::utils::{fuzzy, REGEX_COLLECTION};

pub type CardAndImage = (FuzzyFound, (Option<Vec<u8>>, Option<Vec<u8>>));

pub struct MTG {
    http_client: reqwest::Client,
    images: ImageFetcher,
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

        Self {
            http_client,
            images,
        }
    }

    pub async fn parse_message(&'a self, msg: &'a str) -> Vec<Option<CardAndImage>> {
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
        fuzzy::best_jaro_match_name_fuzzy_found(&normalised_name, potentials)
    }

    async fn search_set_abbreviation(
        &self,
        abbreviation: &str,
        normalised_name: &str,
    ) -> Option<FuzzyFound> {
        let set_name = PSQL::get()?
            .set_name_from_abbreviation(&abbreviation)
            .await?;
        let potentials = PSQL::get()?
            .fuzzy_search_set(&set_name, &normalised_name)
            .await?;
        fuzzy::best_jaro_match_name_fuzzy_found(&normalised_name, potentials)
    }

    async fn search_set_name(
        &self,
        normalised_set_name: &str,
        normalised_name: &str,
    ) -> Option<FuzzyFound> {
        let potentials = PSQL::get()?
            .fuzzy_search_set_name(normalised_set_name)
            .await?;
        let set_name = fuzzy::best_jaro_match(&normalised_set_name, potentials)?;
        let potentials = PSQL::get()?
            .fuzzy_search_set(&set_name, &normalised_name)
            .await?;
        fuzzy::best_jaro_match_name_fuzzy_found(&normalised_name, potentials)
    }

    async fn search_artist(&self, artist: &str, normalised_name: &str) -> Option<FuzzyFound> {
        let potentials = PSQL::get()?.fuzzy_search_for_artist(artist).await?;
        let best_artist = fuzzy::best_jaro_match(&artist, potentials)?;
        let potentials = PSQL::get()?
            .fuzzy_search_artist(&best_artist, &normalised_name)
            .await?;

        fuzzy::best_jaro_match_name_fuzzy_found(&normalised_name, potentials)
    }

    async fn find_card(&self, query: QueryParams) -> Option<CardAndImage> {
        let start = Instant::now();

        let found_card = if let Some(set_code) = &query.set_code {
            self.search_set_abbreviation(&set_code, &query.name).await?
        } else if let Some(set_name) = &query.set_name {
            self.search_set_name(&set_name, &query.name).await?
        } else if let Some(artist) = &query.artist {
            self.search_artist(&artist, &query.name).await?
        } else {
            self.search_distinct_cards(&query.name).await?
        };

        log::info!(
            "Found match for query '{}' in {} ms",
            &query.name,
            start.elapsed().as_millis()
        );

        let images = self.images.fetch(&found_card).await;

        Some((found_card, images))
    }
}
