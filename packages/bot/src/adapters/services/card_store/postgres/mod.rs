mod queries;

use crate::adapters::services::card_store::postgres::queries::{ALL_PRINTS, CARD_FROM_ID, FUZZY_SEARCH_CARD_AND_ARTIST, FUZZY_SEARCH_CARD_AND_SET_NAME, FUZZY_SEARCH_DISTINCT_CARDS, FUZZY_SEARCH_SET_NAME, NORMALISED_SET_NAME, RANDOM_CARD, RANDOM_SET_CARD, SIMILAR_CARDS_FROM};
use contracts::card::Card;
use contracts::set::Set;
use crate::ports::services::card_store::CardStore;
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
            Ok(rows) => Some(rows.into_iter().map(|row| card_from(&row)).collect()),
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
            Ok(rows) => Some(rows.into_iter().map(|row| card_from(&row)).collect()),
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
            Ok(rows) => Some(rows.into_iter().map(|row| card_from(&row)).collect()),
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
            Ok(row) => Some(card_from(&row)),
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
            Ok(row) => Some(card_from(&row)),
        }
    }

    async fn all_prints(&self, oracle_id: &Uuid) -> Option<Vec<Set>> {
        match sqlx::query(ALL_PRINTS)
            .bind(oracle_id)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed search all prints fetch - {why}");
                None
            }
            Ok(rows) => Some(rows.into_iter().map(|row| set_from(&row)).collect()),
        }
    }

    async fn fetch_card_by_id(&self, id: &Uuid) -> Option<Card> {
        match sqlx::query(CARD_FROM_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed card fetch - {why}");
                None
            }
            Ok(row) => Some(card_from(&row)),
        }
    }

    async fn similar_cards(&self, normalised_name: &str) -> Option<Vec<Card>> {
        match sqlx::query(SIMILAR_CARDS_FROM)
            .bind(normalised_name)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed search all prints fetch - {why}");
                None
            }
            Ok(rows) => Some(rows.into_iter().map(|row| card_from(&row)).collect()),
        }
    }
}

fn set_from(row: &PgRow) -> Set {
    Set::new(
        row.get::<Uuid, &str>("card_id"),
        row.get::<String, &str>("set_name"),
    )
}

fn card_from(row: &PgRow) -> Card {
        Card::new(
            row.get::<Uuid, &str>("front_id"),
            row.get::<String, &str>("front_name"),
            row.get::<String, &str>("front_normalised_name"),
            row.get::<Uuid, &str>("front_oracle_id"),
            row.get::<String, &str>("front_scryfall_url"),
            row.get::<Uuid, &str>("front_image_id"),
            row.get::<Option<Uuid>, &str>("front_illustration_id"),
            row.get::<String, &str>("front_mana_cost"),
            row.get::<Vec<String>, &str>("front_colour_identity"),
            row.get::<Option<String>, &str>("front_power"),
            row.get::<Option<String>, &str>("front_toughness"),
            row.get::<Option<String>, &str>("front_loyalty"),
            row.get::<Option<String>, &str>("front_defence"),
            row.get::<String, &str>("front_type_line"),
            row.get::<String, &str>("front_oracle_text"),
            row.get::<Option<Uuid>, &str>("back_id"),
            row.get::<String, &str>("artist"),
            row.get::<String, &str>("set_name"),
            )
    }
