use log;
use serenity::builder::CreateAttachment;
use serenity::futures::future::join_all;
use tokio::time::Instant;

use crate::db::Psql;
use crate::mtg::db::{FuzzyFound, QueryParams};
use crate::mtg::images::ImageFetcher;
use crate::utils;
use crate::utils::{fuzzy, fuzzy_match_set_name, REGEX_COLLECTION};

pub type CardAndImage = (
    FuzzyFound,
    (Option<CreateAttachment>, Option<CreateAttachment>),
);

pub struct MTG {}

impl MTG {
    pub async fn new() -> Self {
        Self {}
    }

    pub async fn parse_message(&self, msg: &str) -> Vec<Option<CardAndImage>> {
        join_all(
            REGEX_COLLECTION
                .cards
                .captures_iter(msg)
                .filter_map(|capture| Some(self.find_card(QueryParams::from(capture)?))),
        )
        .await
    }

    async fn search_distinct_cards(&self, normalised_name: &str) -> Option<FuzzyFound> {
        let potentials = Psql::get()?.fuzzy_search_distinct(normalised_name).await?;
        fuzzy::winkliest_match(normalised_name, potentials)
    }

    async fn search_set_abbreviation(
        &self,
        abbreviation: &str,
        normalised_name: &str,
    ) -> Option<FuzzyFound> {
        let set_name = utils::set_from_abbreviation(abbreviation).await?;
        let potentials = Psql::get()?
            .fuzzy_search_set(&set_name, normalised_name)
            .await?;
        fuzzy::winkliest_match(normalised_name, potentials)
    }

    async fn search_set_name(
        &self,
        normalised_set_name: &str,
        normalised_name: &str,
    ) -> Option<FuzzyFound> {
        let set_name = fuzzy_match_set_name(normalised_set_name).await?;
        let potentials = Psql::get()?
            .fuzzy_search_set(&set_name, normalised_name)
            .await?;
        fuzzy::winkliest_match(normalised_name, potentials)
    }

    async fn search_artist(&self, artist: &str, normalised_name: &str) -> Option<FuzzyFound> {
        let potentials = Psql::get()?.fuzzy_search_for_artist(artist).await?;
        let best_artist = fuzzy::winkliest_match(artist, potentials)?;
        let potentials = Psql::get()?
            .fuzzy_search_artist(&best_artist, normalised_name)
            .await?;

        fuzzy::winkliest_match(normalised_name, potentials)
    }

    async fn find_card(&self, query: QueryParams) -> Option<CardAndImage> {
        let start = Instant::now();

        let found_card = if let Some(set_code) = &query.set_code {
            self.search_set_abbreviation(set_code, &query.name).await?
        } else if let Some(set_name) = &query.set_name {
            self.search_set_name(set_name, &query.name).await?
        } else if let Some(artist) = &query.artist {
            self.search_artist(artist, &query.name).await?
        } else {
            self.search_distinct_cards(&query.name).await?
        };

        log::info!(
            "Found match for query '{}' in {} ms",
            &query.name,
            start.elapsed().as_millis()
        );

        let images = ImageFetcher::get()?.fetch(&found_card).await;

        Some((found_card, images))
    }
}
