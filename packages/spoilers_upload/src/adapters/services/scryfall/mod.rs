mod data;

use crate::adapters::services::scryfall::data::ScryfallData;
use crate::ports::source::CardSource;
use async_trait::async_trait;
use contracts::card::Card;
use contracts::set::Set;
use reqwest::Client;
use std::env;
use crate::adapters::services::scryfall::data::card::ScryfallCard;

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

    async fn recent_sets(&self) -> Vec<Set> {
        let url = format!("{}/sets", self.base_url);

        let response: ScryfallData<Set> = self
            .client
            .get(&url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let today = time::OffsetDateTime::now_utc().date();
        let threshold = today - time::Duration::days(7);

        response.data.into_iter().filter_map(|set| {
            if set.released_at >= threshold && set.card_count > 0 {
                Some(set)
            } else {
                None
            }
        }).collect()
    }
}

#[async_trait]
impl CardSource for Scryfall {
    async fn get_recent_cards(&self) -> Vec<Card> {
        let sets = self.recent_sets().await;
        let mut cards: Vec<ScryfallCard> = Vec::new();

        for set in sets {
            let mut page_number = 1;
            let mut has_more = true;
            while has_more {
                let url = format!("{}/cards/search", self.base_url);
                let response: ScryfallData<ScryfallCard> = self.client
                    .get(&url)
                    .query(&[("q", format!("e:{}", set.abbreviation)), ("page", page_number.to_string())])
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                cards.extend(response.data);

                if response.has_more {
                    page_number += 1;
                } else {
                    has_more = false;
                }
            }
        }

        vec![]
    }
}
