use log;
use serenity::builder::CreateAttachment;
use serenity::futures::future::join_all;
use tokio::time::Instant;

use crate::dbs::psql::Psql;
use crate::mtg::db::{FuzzyFound, QueryParams};
use crate::mtg::images::ImageFetcher;
use crate::utils::{fuzzy, REGEX_COLLECTION};

pub type CardAndImage = (
    FuzzyFound,
    (Option<CreateAttachment>, Option<CreateAttachment>),
);

pub async fn parse_message(msg: &str) -> Vec<Option<CardAndImage>> {
    join_all(
        REGEX_COLLECTION
            .cards
            .captures_iter(msg)
            .filter_map(|capture| Some(find_card(QueryParams::from(capture)?))),
    )
    .await
}

async fn search_distinct_cards(normalised_name: &str) -> Option<FuzzyFound> {
    let potentials = Psql::get()?.fuzzy_search_distinct(normalised_name).await?;
    fuzzy::winkliest_match(normalised_name, potentials)
}

async fn search_set_abbreviation(
    abbreviation: &str,
    normalised_name: &str,
) -> Option<FuzzyFound> {
    let set_name = set_from_abbreviation(abbreviation).await?;
    let potentials = Psql::get()?
        .fuzzy_search_set(&set_name, normalised_name)
        .await?;
    fuzzy::winkliest_match(normalised_name, potentials)
}

async fn search_set_name(
    normalised_set_name: &str,
    normalised_name: &str,
) -> Option<FuzzyFound> {
    let set_name = fuzzy_match_set_name(normalised_set_name).await?;
    let potentials = Psql::get()?
        .fuzzy_search_set(&set_name, normalised_name)
        .await?;
    fuzzy::winkliest_match(normalised_name, potentials)
}

async fn search_artist(artist: &str, normalised_name: &str) -> Option<FuzzyFound> {
    let potentials = Psql::get()?.fuzzy_search_for_artist(artist).await?;
    let best_artist = fuzzy::winkliest_match(artist, potentials)?;
    let potentials = Psql::get()?
        .fuzzy_search_artist(&best_artist, normalised_name)
        .await?;

    fuzzy::winkliest_match(normalised_name, potentials)
}

pub async fn find_card(query: QueryParams) -> Option<CardAndImage> {
    let start = Instant::now();

    let found_card = if let Some(set_code) = &query.set_code {
        search_set_abbreviation(set_code, &query.name).await?
    } else if let Some(set_name) = &query.set_name {
        search_set_name(set_name, &query.name).await?
    } else if let Some(artist) = &query.artist {
        search_artist(artist, &query.name).await?
    } else {
        search_distinct_cards(&query.name).await?
    };

    log::info!(
        "Found match for query '{}' in {} ms",
        &query.name,
        start.elapsed().as_millis()
    );

    let images = ImageFetcher::get()?.fetch(&found_card).await;

    Some((found_card, images))
}

pub async fn fuzzy_match_set_name(normalised_set_name: &str) -> Option<String> {
    let potentials = Psql::get()?
        .fuzzy_search_set_name(normalised_set_name)
        .await?;
    fuzzy::winkliest_match(normalised_set_name, potentials)
}

pub async fn set_from_abbreviation(abbreviation: &str) -> Option<String> {
    Psql::get()?.set_name_from_abbreviation(abbreviation).await
}
