mod queries;

use crate::card_store::postgres::queries::{
    FUZZY_SEARCH_ARTIST, FUZZY_SEARCH_DISTINCT_CARDS, FUZZY_SEARCH_SET_NAME, NORMALISED_SET_NAME,
    RANDOM_CARD_FROM_DISTINCT,
};
use crate::card_store::CardStore;
use crate::mtg::card::Card;
use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Row};
use std::env;

pub struct Postgres {
    pool: Pool<sqlx::Postgres>,
}

#[async_trait]
impl CardStore for Postgres {
    async fn new() -> Self {
        let uri = env::var("PSQL_URI").expect("Postgres uri wasn't in env vars");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&uri)
            .await
            .expect("Failed Postgres connection");

        Self { pool }
    }

    async fn search(&self, normalised_name: &str) -> Option<Vec<Card>> {
        match sqlx::query(FUZZY_SEARCH_DISTINCT_CARDS)
            .bind(normalised_name)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed fuzzy search distinct cards fetch - {why}");
                None
            }
            Ok(rows) => rows
                .into_iter()
                .map(|row| Card::from(&row).ok())
                .collect(),
        }
    }

    async fn search_artist(&self, artist: &str, normalised_name: &str) -> Option<Vec<Card>> {
        match sqlx::query(&format!(
            r"select * from artist_{} where word_similarity(front_normalised_name, $1) > 0.50;",
            artist.replace(' ', "_")
        ))
        .bind(normalised_name)
        .fetch_all(&self.pool)
        .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(rows) => rows
                .into_iter()
                .map(|row| Card::from(&row).ok())
                .collect(),
        }
    }

    async fn search_set(&self, set_name: &str, normalised_name: &str) -> Option<Vec<Card>> {
        match sqlx::query(&format!(
            r"select * from set_{} where word_similarity(front_normalised_name, $1) > 0.50;",
            set_name.replace(' ', "_")
        ))
        .bind(normalised_name)
        .fetch_all(&self.pool)
        .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(rows) => rows
                .into_iter()
                .map(|row| Card::from(&row).ok())
                .collect(),
        }
    }

    async fn search_for_set_name(&self, normalised_name: &str) -> Option<Vec<String>> {
        match sqlx::query(FUZZY_SEARCH_SET_NAME)
            .bind(normalised_name)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed set name fetch - {why}");
                None
            }
            Ok(row) => row.try_get::<Vec<String>, &str>("array_agg").ok(),
        }
    }

    async fn search_for_artist(&self, normalised_name: &str) -> Option<Vec<String>> {
        match sqlx::query(FUZZY_SEARCH_ARTIST)
            .bind(normalised_name)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed artist fetch - {why}");
                None
            }
            Ok(row) => row.try_get::<Vec<String>, &str>("array_agg").ok(),
        }
    }

    async fn set_name_from_abbreviation(&self, abbreviation: &str) -> Option<String> {
        match sqlx::query(NORMALISED_SET_NAME)
            .bind(abbreviation)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed set name from abbr fetch - {why}");
                None
            }
            Ok(row) => Some(row.get::<String, &str>("normalised_name")),
        }
    }

    async fn random_card(&self) -> Option<Card> {
        match sqlx::query(RANDOM_CARD_FROM_DISTINCT)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed random card fetch - {why}");
                None
            }
            Ok(row) => Card::from(&row).ok(),
        }
    }

    async fn random_card_from_set(&self, set_name: &str) -> Option<Card> {
        match sqlx::query(&format!(
            r"select * from set_{} where front_illustration_id is not null order by random() limit 1;",
            set_name.replace(' ', "_")
        ))
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(row) => Card::from(&row).ok(),
        }
    }
}
