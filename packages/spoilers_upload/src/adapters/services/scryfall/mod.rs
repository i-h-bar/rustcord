mod data;

use crate::adapters::services::scryfall::data::ScryfallData;
use crate::ports::source::CardSource;
use async_trait::async_trait;
use contracts::card::Card;
use data::set::ScryfallSet;
use reqwest::Client;
use std::env;
use std::time::{Duration, Instant};
use crate::adapters::services::scryfall::data::card::ScryfallCard;

struct ScryfallResponse<T> {
    scryfall_data: ScryfallData<T>,
    duration: Duration,
}

pub struct Scryfall {
    base_url: String,
    client: Client,
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
        }
    }

    async fn recent_sets(&self) -> Vec<ScryfallSet> {
        let url = format!("{}/sets", self.base_url);
        let response = self.get::<ScryfallSet>(&url).await;

        let today = time::OffsetDateTime::now_utc().date();
        let threshold = today - time::Duration::days(7);

        response.scryfall_data.data.into_iter().filter_map(|set| {
            if set.released_at >= threshold && set.card_count > 0 {
                Some(set)
            } else {
                None
            }
        }).collect()
    }

    async fn get<T>(&self, url: &str) -> ScryfallResponse<T>
    where T: serde::de::DeserializeOwned
    {
        let start = Instant::now();

        let scryfall_data = self.client
            .get(url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        ScryfallResponse { scryfall_data, duration: start.elapsed() }
    }
}

#[async_trait]
impl CardSource for Scryfall {
    async fn get_recent_cards(&self) -> Vec<Card> {
        let sets = self.recent_sets().await;
        let mut cards: Vec<ScryfallCard> = Vec::new();

        for set in sets {
            let mut url = Some(format!("{}/cards/search?q=e:{}", self.base_url, set.abbreviation));
            while let Some(next_page) = url {
                let response = self.get(&next_page).await;
                cards.extend(response.scryfall_data.data);
                url = response.scryfall_data.next_page;
                let response_time = response.duration.as_millis();
                let sleep_time = Duration::from_millis(500).saturating_sub(response.duration);
                tokio::time::sleep(sleep_time).await;
            }
        }

        vec![]
    }
}
