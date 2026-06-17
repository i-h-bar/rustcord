mod data;
pub mod utils;

use crate::adapters::services::scryfall::data::ScryfallData;
use crate::adapters::services::scryfall::data::card::ScryfallCard;
use crate::ports::image_store::Image;
use crate::ports::source::CardSource;
use crate::ports::storage::{Card, CardInfo, Set};
use async_trait::async_trait;
use data::set::ScryfallSet;
use futures::future;
use reqwest::Client;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

struct ScryfallResponse<T> {
    scryfall_data: ScryfallData<T>,
    duration: Duration,
}

#[derive(Default)]
pub struct Scryfall {
    base_url: String,
    client: Client,
    sets: RwLock<HashMap<Uuid, ScryfallSet>>,
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

    async fn recent_sets(&self) -> Vec<ScryfallSet> {
        let url = format!("{}/sets", self.base_url);
        let response = self.get::<ScryfallSet>(&url).await;

        let today = time::OffsetDateTime::now_utc().date();
        let threshold = today - time::Duration::days(7);

        response
            .scryfall_data
            .data
            .into_iter()
            .filter_map(|set| {
                if set.released_at >= threshold && set.card_count > 0 {
                    Some(set)
                } else {
                    None
                }
            })
            .collect()
    }

    async fn get<T>(&self, url: &str) -> ScryfallResponse<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let start = Instant::now();
        let scryfall_data = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        ScryfallResponse {
            scryfall_data,
            duration: start.elapsed(),
        }
    }
}

#[async_trait]
impl CardSource for Scryfall {
    async fn get_recent_sets(&self) -> Vec<Set> {
        if !self.sets.read().await.is_empty() {
            return self.sets.read().await.values().map(Into::into).collect();
        }

        let sets = self.recent_sets().await;
        self.sets
            .write()
            .await
            .extend(sets.into_iter().map(|set| (set.id, set)));

        self.sets.read().await.values().map(Into::into).collect()
    }

    async fn fetch_cards_for_outdated_sets(&self, sets: &[(Set, u32)]) -> Vec<CardInfo> {
        let mut scryfall_cards: Vec<ScryfallCard> = Vec::new();
        log::info!("Fetching {} outdated sets", sets.len());

        for (set, volume) in sets {
            if *volume == 0 {
                log::info!("Scryfall set is empty for {}", set.name);
                continue;
            }

            let is_outdated = self
                .sets
                .read()
                .await
                .get(&set.id)
                .is_some_and(|s| s.card_count != *volume);

            if !is_outdated {
                log::info!("Storage is up to date for {}", set.name);
                continue;
            }

            log::info!("Fetching cards in {}", set.name);
            let mut url = Some(format!(
                "{}/cards/search?q=e:{}",
                self.base_url, set.abbreviation
            ));
            while let Some(next_page) = url {
                let response = self.get(&next_page).await;
                log::info!(
                    "Fetched {} cards for {}",
                    response.scryfall_data.data.len(),
                    set.name
                );
                scryfall_cards.extend(response.scryfall_data.data);

                url = response.scryfall_data.next_page;
                let sleep_time = Duration::from_millis(500).saturating_sub(response.duration);
                tokio::time::sleep(sleep_time).await;
            }
        }

        scryfall_cards
            .into_iter()
            .filter_map(ScryfallCard::into_storage_records)
            .flatten()
            .collect()
    }

    async fn get_image(&self, card: &CardInfo) -> Image {
        let response = self
            .client
            .get(&card.image.scryfall_url)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        Image(card.image.id, response.into())
    }
}
