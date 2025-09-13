mod queries;

use crate::adapters::card_store::postgres::queries::{
    FUZZY_SEARCH_CARD_AND_ARTIST, FUZZY_SEARCH_CARD_AND_SET_NAME, FUZZY_SEARCH_DISTINCT_CARDS,
    FUZZY_SEARCH_SET_NAME, NORMALISED_SET_NAME, RANDOM_CARD, RANDOM_SET_CARD,
};
use crate::adapters::card_store::CardStore;
use crate::domain::card::Card;
use async_trait::async_trait;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{Pool, Row};
use std::env;
use uuid::Uuid;

pub struct Postgres {
    pool: Pool<sqlx::Postgres>,
}

#[async_trait]
impl CardStore for Postgres {
    async fn create() -> Self {
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
            Ok(rows) => Some(rows.into_iter().map(|row| Card::from(&row)).collect()),
        }
    }

    async fn search_artist(&self, artist: &str, normalised_name: &str) -> Option<Vec<Card>> {
        match sqlx::query(FUZZY_SEARCH_CARD_AND_ARTIST)
            .bind(normalised_name)
            .bind(artist)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(rows) => Some(rows.into_iter().map(|row| Card::from(&row)).collect()),
        }
    }

    async fn search_set(&self, set_name: &str, normalised_name: &str) -> Option<Vec<Card>> {
        match sqlx::query(FUZZY_SEARCH_CARD_AND_SET_NAME)
            .bind(normalised_name)
            .bind(set_name)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(rows) => Some(rows.into_iter().map(|row| Card::from(&row)).collect()),
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
        match sqlx::query(RANDOM_CARD).fetch_one(&self.pool).await {
            Err(why) => {
                log::warn!("Failed random card fetch - {why}");
                None
            }
            Ok(row) => Some(Card::from(&row)),
        }
    }

    async fn random_card_from_set(&self, set_name: &str) -> Option<Card> {
        match sqlx::query(RANDOM_SET_CARD)
            .bind(set_name)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(row) => Some(Card::from(&row)),
        }
    }
}

impl Card {
    pub fn from(row: &PgRow) -> Self {
        Self {
            front_name: row.get::<String, &str>("front_name"),
            front_normalised_name: row.get::<String, &str>("front_normalised_name"),
            front_scryfall_url: row.get::<String, &str>("front_scryfall_url"),
            front_image_id: row.get::<Uuid, &str>("front_image_id"),
            front_illustration_id: row.get::<Option<Uuid>, &str>("front_illustration_id"),
            front_mana_cost: row.get::<String, &str>("front_mana_cost"),
            front_colour_identity: row.get::<Vec<String>, &str>("front_colour_identity"),
            front_power: row.get::<Option<String>, &str>("front_power"),
            front_toughness: row.get::<Option<String>, &str>("front_toughness"),
            front_loyalty: row.get::<Option<String>, &str>("front_loyalty"),
            front_defence: row.get::<Option<String>, &str>("front_defence"),
            front_type_line: row.get::<String, &str>("front_type_line"),
            front_oracle_text: row.get::<String, &str>("front_oracle_text"),
            back_name: row.get::<Option<String>, &str>("back_name"),
            back_scryfall_url: row.get::<Option<String>, &str>("back_scryfall_url"),
            back_image_id: row.get::<Option<Uuid>, &str>("back_image_id"),
            back_illustration_id: row.get::<Option<Uuid>, &str>("back_illustration_id"),
            back_mana_cost: row.get::<Option<String>, &str>("back_mana_cost"),
            back_colour_identity: row.get::<Option<Vec<String>>, &str>("back_colour_identity"),
            back_power: row.get::<Option<String>, &str>("back_power"),
            back_toughness: row.get::<Option<String>, &str>("back_toughness"),
            back_loyalty: row.get::<Option<String>, &str>("back_loyalty"),
            back_defence: row.get::<Option<String>, &str>("back_defence"),
            back_type_line: row.get::<Option<String>, &str>("back_type_line"),
            back_oracle_text: row.get::<Option<String>, &str>("back_oracle_text"),
            artist: row.get::<String, &str>("artist"),
            set_name: row.get::<String, &str>("set_name"),
        }
    }
}
