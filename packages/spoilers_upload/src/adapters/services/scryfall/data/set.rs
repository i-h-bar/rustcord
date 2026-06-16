use crate::ports::storage::Set;
use serde::{Deserialize, Serialize};
use time::Date;
use time::serde::format_description;
use uuid::Uuid;

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

impl From<ScryfallSet> for Set {
    fn from(val: ScryfallSet) -> Self {
        let normalised_name = contracts::normalise::normalise(&val.name);

        Set {
            id: val.id,
            name: val.name,
            abbreviation: val.abbreviation,
            normalised_name,
        }
    }
}

impl From<&ScryfallSet> for Set {
    fn from(val: &ScryfallSet) -> Self {
        let normalised_name = contracts::normalise::normalise(&val.name);

        Set {
            id: val.id,
            name: val.name.clone(),
            abbreviation: val.abbreviation.clone(),
            normalised_name,
        }
    }
}
