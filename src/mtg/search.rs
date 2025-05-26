use crate::dbs::psql::Psql;
use crate::mtg::card::FuzzyFound;
use crate::mtg::images::IMAGE_FETCHER;
use crate::utils;
use crate::utils::parse::{ParseError, ResolveOption};
use crate::utils::{fuzzy, REGEX_COLLECTION};
use log;
use regex::Captures;
use serenity::all::ResolvedValue;
use serenity::builder::CreateAttachment;
use serenity::futures::future::join_all;
use tokio::time::Instant;

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
    fuzzy::winkliest_match(&normalised_name, potentials)
}

async fn search_set_abbreviation(abbreviation: &str, normalised_name: &str) -> Option<FuzzyFound> {
    let set_name = set_from_abbreviation(abbreviation).await?;
    let potentials = Psql::get()?
        .fuzzy_search_set(&set_name, normalised_name)
        .await?;
    fuzzy::winkliest_match(&normalised_name, potentials)
}

async fn search_set_name(normalised_set_name: &str, normalised_name: &str) -> Option<FuzzyFound> {
    let set_name = fuzzy_match_set_name(normalised_set_name).await?;
    let potentials = Psql::get()?
        .fuzzy_search_set(&set_name, normalised_name)
        .await?;
    fuzzy::winkliest_match(&normalised_name, potentials)
}

async fn search_artist(artist: &str, normalised_name: &str) -> Option<FuzzyFound> {
    let potentials = Psql::get()?.fuzzy_search_for_artist(artist).await?;
    let best_artist = fuzzy::winkliest_match(&artist, potentials)?;
    let potentials = Psql::get()?
        .fuzzy_search_artist(&best_artist, normalised_name)
        .await?;

    fuzzy::winkliest_match(&normalised_name, potentials)
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

    let images = IMAGE_FETCHER.fetch(&found_card).await;

    Some((found_card, images))
}

pub async fn fuzzy_match_set_name(normalised_set_name: &str) -> Option<String> {
    let potentials = Psql::get()?
        .fuzzy_search_set_name(normalised_set_name)
        .await?;
    fuzzy::winkliest_match(&normalised_set_name, potentials)
}

pub async fn set_from_abbreviation(abbreviation: &str) -> Option<String> {
    Psql::get()?.set_name_from_abbreviation(abbreviation).await
}

pub struct QueryParams {
    name: String,
    set_code: Option<String>,
    set_name: Option<String>,
    artist: Option<String>,
}

impl QueryParams {
    fn from(capture: Captures<'_>) -> Option<Self> {
        let raw_name = capture.get(1)?.as_str();
        let name = utils::normalise(raw_name);
        let (set_code, set_name) = match capture.get(4) {
            Some(set) => {
                let set = set.as_str();
                if set.chars().count() < 5 {
                    (Some(utils::normalise(set)), None)
                } else {
                    (None, Some(utils::normalise(set)))
                }
            }
            None => (None, None),
        };

        let artist = capture
            .get(7)
            .map(|artist| utils::normalise(artist.as_str()));

        Some(Self {
            name,
            artist,
            set_code,
            set_name,
        })
    }
}

impl ResolveOption for QueryParams {
    fn resolve(options: Vec<(&str, ResolvedValue)>) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        let mut card_name = None;
        let mut set_name = None;
        let mut set_code = None;
        let mut artist = None;

        for (name, value) in options {
            match name {
                "name" => {
                    card_name = match value {
                        ResolvedValue::String(card) => Some(card.to_string()),
                        _ => return Err(ParseError::new("Name was not a string")),
                    }
                }
                "set" => {
                    let set = match value {
                        ResolvedValue::String(set) => set.to_string(),
                        _ => return Err(ParseError::new("Name was not a string")),
                    };
                    if set.chars().count() < 5 {
                        set_code = Some(set);
                    } else {
                        set_name = Some(set);
                    }
                }
                "artist" => {
                    artist = match value {
                        ResolvedValue::String(artist) => Some(artist.to_string()),
                        _ => return Err(ParseError::new("Artist was not a string")),
                    }
                }
                _ => {}
            }
        }

        let Some(name) = card_name else {
            return Err(ParseError::new("No name found in query params"));
        };

        Ok(Self {
            name,
            set_name,
            set_code,
            artist,
        })
    }
}
