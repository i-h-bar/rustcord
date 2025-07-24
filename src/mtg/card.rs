use crate::utils::colours::get_colour_identity;
use crate::utils::emoji::add_emoji;
use crate::utils::fuzzy::ToChars;
use crate::utils::{italicise_reminder_text, REGEX_COLLECTION};
use regex::Captures;
use serde::{Deserialize, Serialize};
use serenity::all::{CreateEmbed, CreateEmbedFooter};
use sqlx::postgres::PgRow;
use sqlx::{Error, Row};
use std::str::Chars;
use tokio::time::Instant;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct FuzzyFound {
    pub front_name: String,
    pub front_normalised_name: String,
    pub front_scryfall_url: String,
    pub front_image_id: Uuid,
    pub front_illustration_id: Option<Uuid>,
    pub front_mana_cost: String,
    pub front_colour_identity: Vec<String>,
    pub front_power: Option<String>,
    pub front_toughness: Option<String>,
    pub front_loyalty: Option<String>,
    pub front_defence: Option<String>,
    pub front_type_line: String,
    pub front_oracle_text: String,
    pub back_name: Option<String>,
    pub back_scryfall_url: Option<String>,
    pub back_image_id: Option<Uuid>,
    pub back_illustration_id: Option<Uuid>,
    pub back_mana_cost: Option<String>,
    pub back_colour_identity: Option<Vec<String>>,
    pub back_power: Option<String>,
    pub back_toughness: Option<String>,
    pub back_loyalty: Option<String>,
    pub back_defence: Option<String>,
    pub back_type_line: Option<String>,
    pub back_oracle_text: Option<String>,
    pub artist: String,
    pub set_name: String,
}

impl FuzzyFound {
    #[allow(clippy::missing_errors_doc)]
    pub fn from(row: &PgRow) -> Result<Self, Error> {
        Ok(Self {
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
        })
    }

    #[must_use]
    pub fn image_ids(&self) -> (&Uuid, Option<&Uuid>) {
        (&self.front_image_id, self.back_image_id.as_ref())
    }

    #[must_use]
    pub fn front_image_id(&self) -> &Uuid {
        &self.front_image_id
    }

    #[must_use]
    pub fn back_image_id(&self) -> Option<&Uuid> {
        self.back_image_id.as_ref()
    }

    #[must_use]
    pub fn front_illustration_id(&self) -> Option<&Uuid> {
        self.front_illustration_id.as_ref()
    }

    #[must_use]
    pub fn illustration_ids(&self) -> (Option<&Uuid>, Option<&Uuid>) {
        (
            self.front_illustration_id.as_ref(),
            self.back_illustration_id.as_ref(),
        )
    }

    pub fn to_embed(self) -> (CreateEmbed, Option<CreateEmbed>) {
        let start = Instant::now();

        let stats = if let Some(power) = self.front_power {
            let toughness = self.front_toughness.unwrap_or_else(|| "0".to_string());
            format!("\n\n{power}/{toughness}")
        } else if let Some(loyalty) = self.front_loyalty {
            format!("\n\n{loyalty}")
        } else if let Some(defence) = self.front_defence {
            format!("\n\n{defence}")
        } else {
            String::new()
        };

        let front_oracle_text = REGEX_COLLECTION
            .symbols
            .replace_all(&self.front_oracle_text, |cap: &Captures| add_emoji(cap));
        let front_oracle_text = italicise_reminder_text(&front_oracle_text);

        let rules_text = format!("{}\n\n{}{}", self.front_type_line, front_oracle_text, stats);
        let mana_cost = REGEX_COLLECTION
            .symbols
            .replace_all(&self.front_mana_cost, |cap: &Captures| add_emoji(cap));
        let title = format!("{}        {}", self.front_name, mana_cost);

        let front = CreateEmbed::default()
            .attachment(format!("{}.png", self.front_image_id))
            .url(self.front_scryfall_url)
            .title(title)
            .description(rules_text)
            .colour(get_colour_identity(&self.front_colour_identity))
            .footer(CreateEmbedFooter::new(format!("🖌️ - {}", self.artist)));

        let back = if let Some(name) = self.back_name {
            let stats = if let Some(power) = self.back_power {
                let toughness = self.back_toughness.unwrap_or_else(|| "0".to_string());
                format!("\n\n{power}/{toughness}")
            } else if let Some(loyalty) = self.back_loyalty {
                format!("\n\n{loyalty}")
            } else if let Some(defence) = self.back_defence {
                format!("\n\n{defence}")
            } else {
                String::new()
            };
            let back_oracle_text = self.back_oracle_text.unwrap_or_default();
            let back_oracle_text = REGEX_COLLECTION
                .symbols
                .replace_all(&back_oracle_text, |cap: &Captures| add_emoji(cap));
            let back_oracle_text = italicise_reminder_text(&back_oracle_text);

            let back_rules_text = format!(
                "{}\n\n{}{}",
                self.back_type_line.unwrap_or_default(),
                back_oracle_text,
                stats
            );
            let title = if let Some(mana_cost) = self.back_mana_cost {
                let mana_cost = REGEX_COLLECTION
                    .symbols
                    .replace_all(&mana_cost, |cap: &Captures| add_emoji(cap));
                format!("{name}        {mana_cost}")
            } else {
                name
            };

            let url = self.back_scryfall_url.unwrap_or_default();
            Some(
                CreateEmbed::default()
                    .attachment(format!("{}.png", self.back_image_id.unwrap_or_default()))
                    .url(url)
                    .title(title)
                    .description(back_rules_text)
                    .colour(get_colour_identity(
                        &self.back_colour_identity.unwrap_or_default(),
                    ))
                    .footer(CreateEmbedFooter::new(format!("🖌️ - {}", self.artist))),
            )
        } else {
            None
        };

        log::info!("Format lifetime: {} us", start.elapsed().as_micros());

        (front, back)
    }

    pub fn to_game_embed(&self, multiplier: usize, guesses: usize) -> CreateEmbed {
        let mut embed = CreateEmbed::default()
            .attachment(format!(
                "{}.png",
                self.front_illustration_id.unwrap_or_default()
            ))
            .title("????")
            .description("????")
            .footer(CreateEmbedFooter::new(format!("🖌️ - {}", self.artist)));

        if guesses > multiplier {
            let mana_cost = REGEX_COLLECTION
                .symbols
                .replace_all(&self.front_mana_cost, |cap: &Captures| add_emoji(cap));
            let title = format!("????        {mana_cost}");
            embed = embed
                .title(title)
                .colour(get_colour_identity(&self.front_colour_identity));
        }

        if guesses > multiplier * 2 {
            embed = embed.description(self.rules_text());
        }

        embed
    }

    fn rules_text(&self) -> String {
        let stats = if let Some(power) = self.front_power.clone() {
            let toughness = self
                .front_toughness
                .clone()
                .unwrap_or_else(|| "0".to_string());
            format!("\n\n{power}/{toughness}")
        } else if let Some(loyalty) = self.front_loyalty.clone() {
            format!("\n\n{loyalty}")
        } else if let Some(defence) = self.front_defence.clone() {
            format!("\n\n{defence}")
        } else {
            String::new()
        };

        let front_oracle_text = REGEX_COLLECTION
            .symbols
            .replace_all(&self.front_oracle_text, |cap: &Captures| add_emoji(cap));
        let front_oracle_text = italicise_reminder_text(&front_oracle_text);

        format!("{}\n\n{}{}", self.front_type_line, front_oracle_text, stats)
    }

    #[must_use]
    pub fn set_name(&self) -> &str {
        &self.set_name
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
