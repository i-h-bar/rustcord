use crate::app::App;
use crate::card_store::CardStore;
use crate::image_store::{ImageStore, Images};
use crate::mtg::card::FuzzyFound;
use crate::query::QueryParams;
use crate::utils::{fuzzy, REGEX_COLLECTION};
use serenity::futures::future::join_all;
use tokio::time::Instant;

pub type CardAndImage = (FuzzyFound, Images);

impl<IS, CS> App<IS, CS>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
{
    pub async fn parse_message(&self, msg: &str) -> Vec<Option<CardAndImage>> {
        join_all(
            REGEX_COLLECTION
                .cards
                .captures_iter(msg)
                .filter_map(|capture| Some(self.find_card(QueryParams::from(&capture)?))),
        )
        .await
    }

    async fn search_distinct_cards(&self, normalised_name: &str) -> Option<FuzzyFound> {
        let potentials = self.card_store.search(normalised_name).await?;
        fuzzy::winkliest_match(&normalised_name, potentials)
    }

    pub async fn search_set_abbreviation(
        &self,
        abbreviation: &str,
        normalised_name: &str,
    ) -> Option<FuzzyFound> {
        let set_name = self.set_from_abbreviation(abbreviation).await?;
        let potentials = self
            .card_store
            .search_set(&set_name, normalised_name)
            .await?;
        fuzzy::winkliest_match(&normalised_name, potentials)
    }

    async fn search_set_name(
        &self,
        normalised_set_name: &str,
        normalised_name: &str,
    ) -> Option<FuzzyFound> {
        let set_name = self.fuzzy_match_set_name(normalised_set_name).await?;
        let potentials = self
            .card_store
            .search_set(&set_name, normalised_name)
            .await?;
        fuzzy::winkliest_match(&normalised_name, potentials)
    }

    async fn search_artist(&self, artist: &str, normalised_name: &str) -> Option<FuzzyFound> {
        let potentials = self.card_store.search_for_artist(artist).await?;
        let best_artist = fuzzy::winkliest_match(&artist, potentials)?;
        let potentials = self
            .card_store
            .search_artist(&best_artist, normalised_name)
            .await?;

        fuzzy::winkliest_match(&normalised_name, potentials)
    }

    pub async fn find_card(&self, query: QueryParams) -> Option<CardAndImage> {
        let start = Instant::now();

        let found_card = if let Some(set_code) = query.set_code() {
            self.search_set_abbreviation(set_code, query.name()).await?
        } else if let Some(set_name) = query.set_name() {
            self.search_set_name(set_name, query.name()).await?
        } else if let Some(artist) = query.artist() {
            self.search_artist(artist, query.name()).await?
        } else {
            self.search_distinct_cards(query.name()).await?
        };

        log::info!(
            "Found match for query '{}' in {} ms",
            query.name(),
            start.elapsed().as_millis()
        );

        let images = self.image_store.fetch(&found_card).await.ok()?;

        Some((found_card, images))
    }

    pub async fn fuzzy_match_set_name(&self, normalised_set_name: &str) -> Option<String> {
        let potentials = self
            .card_store
            .search_for_set_name(normalised_set_name)
            .await?;
        fuzzy::winkliest_match(&normalised_set_name, potentials)
    }

    pub async fn set_from_abbreviation(&self, abbreviation: &str) -> Option<String> {
        self.card_store
            .set_name_from_abbreviation(abbreviation)
            .await
    }
}
