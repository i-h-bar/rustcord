mod data;
pub mod utils;

use crate::adapters::services::scryfall::data::ScryfallData;
use crate::adapters::services::scryfall::data::card::ScryfallCard;
use crate::domain::utils::emoji::normalise_name;
use crate::ports::emoji::{Emoji, EmojiImage, EmojiMetaData};
use crate::ports::image_store::{Illustration, Image};
use crate::ports::source::CardSource;
use crate::ports::storage::{CardInfo, Set};
use async_trait::async_trait;
use data::set::ScryfallSet;
use futures::future;
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use reqwest::{Client, Response};
use std::collections::{HashMap, HashSet};
use std::env;
use std::num::NonZeroU32;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

fn check_if_rate_limited(resp: &Response) {
    if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        log::warn!("Rate limited by Scryfall");
        std::process::exit(1);
    }
}

#[derive(Error, Debug)]
enum ScryfallError {
    #[error("Http error {0}")]
    HTTPError(#[from] reqwest::Error),
    #[error("Error parsing response from scryfall")]
    ParseError,
}

type Limiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;
type ScryfallResult<T> = Result<T, ScryfallError>;

pub struct Scryfall {
    base_url: String,
    client: Client,
    sets: RwLock<HashMap<Uuid, ScryfallSet>>,
    low_limiter: Arc<Limiter>,
    high_limiter: Arc<Limiter>,
}

impl Default for Scryfall {
    fn default() -> Self {
        let low_quota = Quota::per_second(NonZeroU32::new(2).unwrap());
        let high_quota = Quota::per_second(NonZeroU32::new(10).unwrap());
        Self {
            base_url: String::default(),
            client: Client::default(),
            sets: RwLock::default(),
            low_limiter: Arc::new(RateLimiter::direct(low_quota)),
            high_limiter: Arc::new(RateLimiter::direct(high_quota)),
        }
    }
}

impl Scryfall {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(env::var("USER_AGENT").expect("USER_AGENT wasn't in env vars"))
            .build()
            .expect("Failure to creating reqwest client");

        Self {
            base_url: "https://api.scryfall.com".into(),
            client,
            ..Self::default()
        }
    }

    async fn recent_sets(&self) -> ScryfallResult<Vec<ScryfallSet>> {
        let url = format!("{}/sets", self.base_url);
        let response = self.get(&url).await?;

        let today = time::OffsetDateTime::now_utc().date();
        let threshold = today - time::Duration::days(7);

        Ok(response
            .data
            .into_iter()
            .filter_map(|set: ScryfallSet| {
                if set.released_at >= threshold && set.card_count > 0 {
                    Some(set)
                } else {
                    None
                }
            })
            .collect())
    }

    async fn get_resp(&self, url: &str, limiter: &Limiter) -> ScryfallResult<Response> {
        limiter.until_ready().await;
        let resp = self.client.get(url).send().await.map_err(|why| {
            log::warn!("Error getting data from scryfall: {why}");
            ScryfallError::HTTPError(why)
        })?;

        check_if_rate_limited(&resp);

        Ok(resp)
    }

    async fn get_text(&self, url: &str, limiter: &Limiter) -> ScryfallResult<String> {
        let resp = self.get_resp(url, limiter).await?;

        resp.text().await.map_err(|err| {
            log::warn!("Error parsing data from scryfall: {err}");
            ScryfallError::ParseError
        })
    }

    async fn get<T>(&self, url: &str) -> ScryfallResult<ScryfallData<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let resp = self.get_resp(url, &self.low_limiter).await?;

        resp.json().await.map_err(|err| {
            log::warn!("Error parsing data from scryfall: {err}");
            ScryfallError::ParseError
        })
    }
}

#[async_trait]
impl CardSource for Scryfall {
    async fn get_recent_sets(&self) -> Vec<Set> {
        if !self.sets.read().await.is_empty() {
            return self.sets.read().await.values().map(Into::into).collect();
        }

        let Ok(sets) = self.recent_sets().await else {
            return Vec::new();
        };
        self.sets
            .write()
            .await
            .extend(sets.into_iter().map(|set| (set.id, set)));

        self.sets.read().await.values().map(Into::into).collect()
    }

    async fn fetch_cards_for_outdated_sets(&self, sets: &[(Set, HashSet<Uuid>)]) -> Vec<CardInfo> {
        let mut scryfall_cards: Vec<ScryfallCard> = Vec::new();
        log::info!("Fetching {} sets", sets.len());

        'set: for (set, existing_ids) in sets {
            log::info!(
                "Fetching cards in {} ({} already stored)",
                set.name,
                existing_ids.len()
            );
            let mut url = Some(format!(
                "{}/cards/search?q=e:{}",
                self.base_url, set.abbreviation
            ));

            let mut set_cards = Vec::new();
            while let Some(ref next_page) = url {
                let Ok(response) = self.get(next_page).await else {
                    continue 'set;
                };
                set_cards.extend(response.data);
                url = response.next_page;
            }

            set_cards.retain(|c: &ScryfallCard| !existing_ids.contains(&c.id));
            if !set_cards.is_empty() {
                log::info!("Found {} new cards for {}", set_cards.len(), set.name);
                scryfall_cards.extend(set_cards);
            }
        }

        scryfall_cards
            .into_iter()
            .filter_map(ScryfallCard::into_storage_records)
            .flatten()
            .collect()
    }

    async fn get_image(&self, card: &CardInfo) -> Option<Image> {
        let url = &card.image.scryfall_url;
        let resp = self.get_resp(url, &self.high_limiter).await.ok()?;

        let image = match resp.bytes().await {
            Ok(image) => image,
            Err(why) => {
                log::warn!("Error parsing image from scryfall: {why}");
                return None;
            }
        };

        Some(Image(card.image.id, image.into()))
    }

    async fn get_illustration(&self, card: &CardInfo) -> Option<Illustration> {
        let illustration = card.illustration.as_ref()?;
        let resp = self
            .get_resp(&illustration.scryfall_url, &self.high_limiter)
            .await
            .ok()?;

        let image = match resp.bytes().await {
            Ok(image) => image,
            Err(why) => {
                log::warn!("Error parsing image from scryfall: {why}");
                return None;
            }
        };

        Some(Illustration(illustration.id, image.into()))
    }

    async fn fetch_missing_set_symbols(&self, current: &[EmojiMetaData]) -> Vec<Emoji> {
        if current.len() >= self.sets.read().await.len() {
            return vec![];
        }

        let current_sets: HashSet<&str> = current.iter().map(|e| e.name.as_str()).collect();

        future::join_all(
            self.sets
                .read()
                .await
                .values()
                .filter(|s| !current_sets.contains(normalise_name(&s.abbreviation).as_str()))
                .collect::<Vec<&ScryfallSet>>()
                .iter()
                .map(|s| async {
                    let data = self
                        .get_text(&s.icon_svg_uri, &self.high_limiter)
                        .await
                        .ok()?;

                    Some(Emoji {
                        name: s.abbreviation.clone(),
                        image: EmojiImage(data),
                    })
                }),
        )
        .await
        .into_iter()
        .flatten()
        .collect()
    }
}
