mod queries;

use crate::db::PSQL;
use crate::mtg::{CardInfo, FoundCard};
use crate::utils;
use regex::Captures;
use sqlx::postgres::PgRow;
use sqlx::{Error, FromRow, Row};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;
use crate::mtg::db::queries::FUZZY_SEARCH_DISTINCT_CARDS;

pub struct FuzzyFound {
    front_id: Uuid,
    front_name: String,
    front_normalised_name: String,
    front_image_id: Uuid,
    front_mana_cost: String,
    front_power: Option<String>,
    front_toughness: Option<String>,
    front_loyalty: Option<String>,
    front_defence: Option<String>,
    front_type_line: String,
    front_keywords: Vec<String>,
    front_oracle_text: String,
    back_id: Option<Uuid>,
    back_name: Option<String>,
    back_image_id: Option<Uuid>,
    back_mana_cost: Option<String>,
    back_power: Option<String>,
    back_toughness: Option<String>,
    back_loyalty: Option<String>,
    back_defence: Option<String>,
    back_type_line: Option<String>,
    back_keywords: Option<Vec<String>>,
    back_oracle_text: Option<String>,
    release_date: String,
}

impl<'r> FromRow<'r, PgRow> for FuzzyFound {
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        Ok(FuzzyFound {
            front_id: row.get::<Uuid, &str>("front_id"),
            front_name: row.get::<String, &str>("front_name"),
            front_normalised_name: row.get::<String, &str>("front_normalised_name"),
            front_image_id: row.get::<Uuid, &str>("front_image_id"),
            front_mana_cost: row.get::<String, &str>("front_mana_cost"),
            front_power: row.get::<Option<String>, &str>("front_power"),
            front_toughness: row.get::<Option<String>, &str>("front_toughness"),
            front_loyalty: row.get::<Option<String>, &str>("front_loyalty"),
            front_defence: row.get::<Option<String>, &str>("front_defence"),
            front_type_line: row.get::<String, &str>("front_type_line"),
            front_keywords: row.get::<Vec<String>, &str>("front_keywords"),
            front_oracle_text: row.get::<String, &str>("front_oracle_text"),
            back_id: row.get::<Option<Uuid>, &str>("back_id"),
            back_name: row.get::<Option<String>, &str>("back_name"),
            back_image_id: row.get::<Option<Uuid>, &str>("back_image_id"),
            back_mana_cost: row.get::<Option<String>, &str>("back_mana_cost"),
            back_power: row.get::<Option<String>, &str>("back_power"),
            back_toughness: row.get::<Option<String>, &str>("back_toughness"),
            back_loyalty: row.get::<Option<String>, &str>("back_loyalty"),
            back_defence: row.get::<Option<String>, &str>("back_defence"),
            back_type_line: row.get::<Option<String>, &str>("back_type_line"),
            back_keywords: row.get::<Option<Vec<String>>, &str>("back_keywords"),
            back_oracle_text: row.get::<Option<String>, &str>("back_oracle_text"),
            release_date: row.get::<String, &str>("release_date"),
        })
    }
}

impl PSQL {
    pub async fn fuzzy_search_distinct(&self, normalised_name: &str) -> Option<Vec<FuzzyFound>> {
        match sqlx::query(FUZZY_SEARCH_DISTINCT_CARDS)
            .bind(&normalised_name)
            .fetch_all(&self.pool)
            .await {
            Err(why) => {
                log::warn!("Failed card fetch - {why}");
                None
            }
            Ok(rows) => rows.into_iter().map(| row | FuzzyFound::from_row(&row).ok()).collect()
        }
    }
}

pub struct QueryParams<'a> {
    pub name: String,
    pub raw_name: &'a str,
    pub set_code: Option<String>,
    pub set_name: Option<String>,
    pub artist: Option<String>,
}

impl<'a> QueryParams<'a> {
    pub fn from(capture: Captures<'a>) -> Option<Self> {
        let raw_name = capture.get(1)?.as_str();
        let name = utils::normalise(&raw_name);
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

        let artist = match capture.get(7) {
            Some(artist) => Some(utils::normalise(&artist.as_str())),
            None => None,
        };

        Some(Self {
            name,
            raw_name,
            artist,
            set_code,
            set_name,
        })
    }
}
