mod queries;

use crate::db::PSQL;
use crate::emoji::add_emoji;
use crate::mtg::db::queries::{
    FUZZY_SEARCH_ARTIST, FUZZY_SEARCH_DISTINCT_CARDS, FUZZY_SEARCH_SET_NAME, NORMALISED_SET_NAME,
};
use crate::utils;
use crate::utils::colours::get_colour_identity;
use crate::utils::fuzzy::ToChars;
use crate::utils::{italicise_reminder_text, REGEX_COLLECTION};
use regex::Captures;
use serenity::all::CreateEmbed;
use sqlx::postgres::PgRow;
use sqlx::{Error, FromRow, Row};
use std::str::Chars;
use tokio::time::Instant;
use uuid::Uuid;

pub struct FuzzyFound {
    front_name: String,
    front_normalised_name: String,
    front_scryfall_url: String,
    front_image_id: Uuid,
    front_mana_cost: String,
    front_colour_identity: Vec<String>,
    front_power: Option<String>,
    front_toughness: Option<String>,
    front_loyalty: Option<String>,
    front_defence: Option<String>,
    front_type_line: String,
    front_oracle_text: String,
    back_name: Option<String>,
    back_scryfall_url: Option<String>,
    back_image_id: Option<Uuid>,
    back_mana_cost: Option<String>,
    back_colour_identity: Option<Vec<String>>,
    back_power: Option<String>,
    back_toughness: Option<String>,
    back_loyalty: Option<String>,
    back_defence: Option<String>,
    back_type_line: Option<String>,
    back_oracle_text: Option<String>,
}

impl FuzzyFound {
    pub fn image_ids(&self) -> (&Uuid, Option<&Uuid>) {
        (&self.front_image_id, self.back_image_id.as_ref())
    }

    pub fn to_embed(self) -> (CreateEmbed, Option<CreateEmbed>) {
        let start = Instant::now();

        let stats = if let Some(power) = self.front_power {
            let toughness = self.front_toughness.unwrap_or_else(|| "0".to_string());
            format!("\n\n{}/{}", power, toughness)
        } else if let Some(loyalty) = self.front_loyalty {
            format!("\n\n{}", loyalty)
        } else if let Some(defence) = self.front_defence {
            format!("\n\n{}", defence)
        } else {
            "".to_string()
        };

        let front_oracle_text = REGEX_COLLECTION
            .symbols
            .replace_all(&self.front_oracle_text, |cap: &Captures| add_emoji(&cap));
        let front_oracle_text = italicise_reminder_text(&front_oracle_text);

        let rules_text = format!("{}\n\n{}{}", self.front_type_line, front_oracle_text, stats);
        let mana_cost = REGEX_COLLECTION
            .symbols
            .replace_all(&self.front_mana_cost, |cap: &Captures| add_emoji(&cap));
        let title = format!("{}        {}", self.front_name, mana_cost);

        let front = CreateEmbed::default()
            .attachment(format!("{}.png", self.front_image_id.to_string()))
            .url(self.front_scryfall_url)
            .title(title)
            .description(rules_text)
            .colour(get_colour_identity(self.front_colour_identity));

        let back = if let Some(name) = self.back_name {
            let stats = if let Some(power) = self.back_power {
                let toughness = self.back_toughness.unwrap_or_else(|| "0".to_string());
                format!("\n\n{}/{}", power, toughness)
            } else if let Some(loyalty) = self.back_loyalty {
                format!("\n\n{}", loyalty)
            } else if let Some(defence) = self.back_defence {
                format!("\n\n{}", defence)
            } else {
                "".to_string()
            };
            let back_oracle_text = self.back_oracle_text.unwrap_or_else(|| "".to_string());
            let back_oracle_text = REGEX_COLLECTION
                .symbols
                .replace_all(&back_oracle_text, |cap: &Captures| add_emoji(&cap));
            let back_oracle_text = italicise_reminder_text(&back_oracle_text);

            let back_rules_text = format!(
                "{}\n\n{}{}",
                self.back_type_line.unwrap_or_else(|| "".to_string()),
                back_oracle_text,
                stats
            );
            let title = if let Some(mana_cost) = self.back_mana_cost {
                let mana_cost = REGEX_COLLECTION
                    .symbols
                    .replace_all(&mana_cost, |cap: &Captures| add_emoji(&cap));
                format!("{}        {}", name, mana_cost)
            } else {
                name
            };

            let url = self.back_scryfall_url.unwrap_or_else(|| "".to_string());
            Some(
                CreateEmbed::default()
                    .attachment(format!(
                        "{}.png",
                        self.back_image_id
                            .unwrap_or_else(|| Uuid::default())
                            .to_string()
                    ))
                    .url(url)
                    .title(title)
                    .description(back_rules_text)
                    .colour(get_colour_identity(
                        self.back_colour_identity.unwrap_or_else(|| Vec::new()),
                    )),
            )
        } else {
            None
        };

        log::info!("Format lifetime: {} us", start.elapsed().as_micros());

        (front, back)
    }
}

impl ToChars for FuzzyFound {
    fn to_chars(&self) -> Chars<'_> {
        self.front_normalised_name.chars()
    }
}

impl PartialEq<FuzzyFound> for &str {
    fn eq(&self, other: &FuzzyFound) -> bool {
        self == &other.front_normalised_name
    }
}

impl<'r> FromRow<'r, PgRow> for FuzzyFound {
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        Ok(FuzzyFound {
            front_name: row.get::<String, &str>("front_name"),
            front_normalised_name: row.get::<String, &str>("front_normalised_name"),
            front_scryfall_url: row.get::<String, &str>("front_scryfall_url"),
            front_image_id: row.get::<Uuid, &str>("front_image_id"),
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
            back_mana_cost: row.get::<Option<String>, &str>("back_mana_cost"),
            back_colour_identity: row.get::<Option<Vec<String>>, &str>("back_colour_identity"),
            back_power: row.get::<Option<String>, &str>("back_power"),
            back_toughness: row.get::<Option<String>, &str>("back_toughness"),
            back_loyalty: row.get::<Option<String>, &str>("back_loyalty"),
            back_defence: row.get::<Option<String>, &str>("back_defence"),
            back_type_line: row.get::<Option<String>, &str>("back_type_line"),
            back_oracle_text: row.get::<Option<String>, &str>("back_oracle_text"),
        })
    }
}

impl PSQL {
    pub async fn fuzzy_search_distinct(&self, normalised_name: &str) -> Option<Vec<FuzzyFound>> {
        match sqlx::query(FUZZY_SEARCH_DISTINCT_CARDS)
            .bind(&normalised_name)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed fuzzy search distinct cards fetch - {why}");
                None
            }
            Ok(rows) => rows
                .into_iter()
                .map(|row| FuzzyFound::from_row(&row).ok())
                .collect(),
        }
    }

    pub async fn set_name_from_abbreviation(&self, abbreviation: &str) -> Option<String> {
        match sqlx::query(NORMALISED_SET_NAME)
            .bind(&abbreviation)
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

    pub async fn fuzzy_search_set(
        &self,
        set_name: &str,
        normalised_name: &str,
    ) -> Option<Vec<FuzzyFound>> {
        match sqlx::query(&format!(
            r#"
            select  front_name,
                    front_normalised_name,
                    front_scryfall_url,
                    front_image_id,
                    front_mana_cost,
                    front_colour_identity,
                    front_power,
                    front_toughness,
                    front_loyalty,
                    front_defence,
                    front_type_line,
                    front_oracle_text,
                    back_name,
                    back_scryfall_url,
                    back_image_id,
                    back_mana_cost,
                    back_colour_identity,
                    back_power,
                    back_toughness,
                    back_loyalty,
                    back_defence,
                    back_type_line,
                    back_oracle_text
            from set_{} where word_similarity(front_normalised_name, $1) > 0.50;"#,
            set_name.replace(" ", "_")
        ))
        .bind(&normalised_name)
        .fetch_all(&self.pool)
        .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(rows) => rows
                .into_iter()
                .map(|row| FuzzyFound::from_row(&row).ok())
                .collect(),
        }
    }

    pub async fn fuzzy_search_set_name(&self, normalised_name: &str) -> Option<Vec<String>> {
        match sqlx::query(FUZZY_SEARCH_SET_NAME)
            .bind(&normalised_name)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed set name fetch - {why}");
                None
            }
            Ok(row) => {
                if let Ok(row) = row.try_get::<Vec<String>, &str>("array_agg") {
                    Some(row)
                } else {
                    None
                }
            },
        }
    }

    pub async fn fuzzy_search_for_artist(&self, normalised_name: &str) -> Option<Vec<String>> {
        match sqlx::query(FUZZY_SEARCH_ARTIST)
            .bind(&normalised_name)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed artist fetch - {why}");
                None
            }
            Ok(row) => {
                if let Ok(row) = row.try_get::<Vec<String>, &str>("array_agg") {
                    Some(row)
                } else {
                    None
                }
            },
        }
    }

    pub async fn fuzzy_search_artist(
        &self,
        artist: &str,
        normalised_name: &str,
    ) -> Option<Vec<FuzzyFound>> {
        match sqlx::query(&format!(
            r#"
            select  front_name,
                    front_normalised_name,
                    front_scryfall_url,
                    front_image_id,
                    front_mana_cost,
                    front_colour_identity,
                    front_power,
                    front_toughness,
                    front_loyalty,
                    front_defence,
                    front_type_line,
                    front_oracle_text,
                    back_name,
                    back_scryfall_url,
                    back_image_id,
                    back_mana_cost,
                    back_colour_identity,
                    back_power,
                    back_toughness,
                    back_loyalty,
                    back_defence,
                    back_type_line,
                    back_oracle_text
            from artist_{} where word_similarity(front_normalised_name, $1) > 0.50;"#,
            artist.replace(" ", "_")
        ))
        .bind(&normalised_name)
        .fetch_all(&self.pool)
        .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(rows) => rows
                .into_iter()
                .map(|row| FuzzyFound::from_row(&row).ok())
                .collect(),
        }
    }
}

pub struct QueryParams {
    pub name: String,
    pub set_code: Option<String>,
    pub set_name: Option<String>,
    pub artist: Option<String>,
}

impl<'a> QueryParams {
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
            artist,
            set_code,
            set_name,
        })
    }
}
