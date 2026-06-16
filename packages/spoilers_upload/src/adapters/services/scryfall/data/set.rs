use serde::{Deserialize, Serialize};
use time::Date;
use time::serde::format_description;
use uuid::Uuid;
use crate::ports::storage::Set;

format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Deserialize, Serialize)]
pub struct ScryfallSet {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "code")]
    pub abbreviation: String,
    pub icon_svg_uri: String,
    pub card_count: u32,
    #[serde(with = "date_format")]
    pub released_at: Date,
}

impl Into<Set> for ScryfallSet {
    fn into(self) -> Set {
        let normalised_name = contracts::normalise::normalise(&self.name);

        Set {
            id: self.id,
            name: self.name,
            abbreviation: self.abbreviation,
            normalised_name,
        }
    }
}

impl Into<Set> for &ScryfallSet {
    fn into(self) -> Set {
        let normalised_name = contracts::normalise::normalise(&self.name);

        Set {
            id: self.id,
            name: self.name.clone(),
            abbreviation: self.abbreviation.clone(),
            normalised_name,
        }
    }
}