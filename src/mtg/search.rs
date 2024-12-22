use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use log;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serenity::futures::future::join_all;
use tokio::time::Instant;

use crate::db::PSQL;
use crate::mtg::db::QueryParams;
use crate::mtg::{CardFace, FoundCard, ScryfallCard, ScryfallList};
use crate::utils::{fuzzy, REGEX_COLLECTION};

const SCRYFALL: &str = "https://api.scryfall.com/cards/search?unique=prints&q=";

pub struct MTG {
    http_client: reqwest::Client,
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

        Self { http_client }
    }

    pub async fn parse_message(&'a self, msg: &'a str) -> Vec<Option<FoundCard<'a>>> {
        let queries: Vec<_> = REGEX_COLLECTION
            .cards
            .captures_iter(&msg)
            .filter_map(|capture| Some(Arc::new(QueryParams::from(capture)?)))
            .collect();

        let futures = queries
            .into_iter()
            .map(|query| self.find_card(Arc::clone(&query)));

        join_all(futures).await
    }

    async fn find_card(&'a self, query: Arc<QueryParams<'a>>) -> Option<FoundCard<'a>> {
        let start = Instant::now();

        let card = if let Some(fuzzy_found) = PSQL::get()?.fuzzy_fetch(Arc::clone(&query)).await {
            if fuzzy_found.similarity > 0.75 {
                let back = if let Some(other_side) = fuzzy_found.other_side.as_ref() {
                    PSQL::get()?.fetch_backside(&other_side).await
                } else {
                    None
                };

                log::info!("Found match for '{}' from database", query.raw_name);

                FoundCard::existing_card(Arc::clone(&query), fuzzy_found, back)
            } else {
                self.find_from_scryfall(Arc::clone(&query)).await
            }
        } else {
            self.find_from_scryfall(Arc::clone(&query)).await
        };

        log::info!("Found match in {} ms", start.elapsed().as_millis());

        card
    }

    fn determine_best_match<'b>(
        &'a self,
        query: Arc<QueryParams<'a>>,
        cards: &'b ScryfallList,
    ) -> Option<ScryfallCard> {
        let unique_cards: HashSet<String> =
            HashSet::from_iter(cards.data.iter().map(|card| card.name.clone()));
        let best_match = fuzzy::best_match_lev(&query.name, &unique_cards)?;
        let potential_cards: Vec<&ScryfallCard> = cards
            .data
            .iter()
            .filter_map(|card| {
                if &card.name == best_match {
                    Some(card)
                } else {
                    None
                }
            })
            .collect();

        let potential_cards = if let Some(queried_artist) = &query.artist {
            let artists_set: HashSet<String> =
                HashSet::from_iter(potential_cards.iter().map(|card| card.artist.clone()));
            let best_artist = fuzzy::best_match_lev(queried_artist, &artists_set)?;
            potential_cards
                .iter()
                .filter_map(|&card| {
                    if best_artist == &card.artist {
                        Some(card)
                    } else {
                        None
                    }
                })
                .collect::<Vec<&ScryfallCard>>()
        } else {
            potential_cards
        };

        let potential_cards = if let Some(queried_set_name) = &query.set_name {
            let set_name_set: HashSet<String> =
                HashSet::from_iter(potential_cards.iter().map(|&card| card.set_name.clone()));
            let best_set = fuzzy::best_match_lev(&queried_set_name, &set_name_set)?;
            potential_cards
                .iter()
                .filter_map(|&card| {
                    if &card.set_name == best_set {
                        Some(card)
                    } else {
                        None
                    }
                })
                .collect::<Vec<&ScryfallCard>>()
        } else {
            potential_cards
        };

        let potential_cards = if let Some(queried_set_code) = &query.set_code {
            potential_cards
                .iter()
                .filter_map(|&card| {
                    if &card.set == queried_set_code {
                        Some(card)
                    } else {
                        None
                    }
                })
                .collect::<Vec<&ScryfallCard>>()
        } else {
            potential_cards
        };

        Some((**potential_cards.get(0)?).clone())
    }

    async fn fetch_images(
        &self,
        url: &str,
        name: &str,
        back_url: Option<&str>,
    ) -> Option<Vec<Vec<u8>>> {
        if let Some(back) = back_url {
            Some(vec![
                self.query_image(&url, &name).await?,
                self.query_image(&back, &name).await?,
            ])
        } else {
            Some(vec![self.query_image(&url, name).await?])
        }
    }

    pub async fn create_found_card(
        &'a self,
        query: Arc<QueryParams<'a>>,
        card: &ScryfallCard,
        cards: Option<ScryfallList>,
    ) -> Option<FoundCard<'a>> {
        match &card.card_faces {
            Some(card_faces) => {
                let face_0 = <Vec<CardFace> as AsRef<Vec<CardFace>>>::as_ref(card_faces).get(0)?;
                let face_1 = <Vec<CardFace> as AsRef<Vec<CardFace>>>::as_ref(card_faces).get(1)?;

                let images = self
                    .fetch_images(
                        &face_0.image_uris.png,
                        &query.name,
                        Some(&face_1.image_uris.png),
                    )
                    .await?;
                FoundCard::new_2_faced_card(Arc::clone(&query), &card, images, cards)
            }
            None => {
                log::info!(
                    "Matched with - \"{}\". Now searching for image...",
                    card.name
                );
                let image = self
                    .fetch_images(&card.image_uris.as_ref()?.png, &query.name, None)
                    .await?
                    .get(0)?
                    .to_owned();

                Some(FoundCard::new_card(Arc::clone(&query), &card, image, cards))
            }
        }
    }

    async fn find_from_scryfall(&'a self, query: Arc<QueryParams<'a>>) -> Option<FoundCard> {
        let cards = self.search_scryfall_card_data(Arc::clone(&query)).await?;
        let card = self.determine_best_match(Arc::clone(&query), &cards)?;

        let found_card = self
            .create_found_card(Arc::clone(&query), &card, Some(cards))
            .await;

        log::info!("Found match for '{}' from scryfall", query.raw_name);

        found_card
    }

    async fn query_image(&self, url: &str, name: &str) -> Option<Vec<u8>> {
        let Ok(image) = {
            match self.http_client.get(url).send().await {
                Ok(response) => response,
                Err(why) => {
                    log::warn!("Error grabbing image for '{}' because: {}", name, why);
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

        log::info!("Image found for - \"{}\".", name);
        Some(image.to_vec())
    }

    async fn search_scryfall_card_data(&self, query: Arc<QueryParams<'_>>) -> Option<ScryfallList> {
        log::info!("Searching scryfall for '{}'", query.raw_name);
        let response = match self
            .http_client
            .get(format!("{}{}", SCRYFALL, query.name.replace(" ", "+")))
            .send()
            .await
        {
            Ok(response) => response,
            Err(why) => {
                log::warn!("Error searching for '{}' because: {}", query.raw_name, why);
                return None;
            }
        };

        if response.status().is_success() {
            match response.json::<ScryfallList>().await {
                Ok(response) => Some(response),
                Err(why) => {
                    log::warn!("Error getting card from scryfall - {why:?}");
                    None
                }
            }
        } else {
            match response.status().as_u16() {
                404 => {
                    log::info!(
                        "Could not find card from scryfall with the name '{}'",
                        query.raw_name
                    )
                }
                status => {
                    log::warn!(
                        "None 200 response from scryfall: {} when searching for '{}'",
                        status,
                        query.raw_name
                    );
                }
            }

            None
        }
    }
}
