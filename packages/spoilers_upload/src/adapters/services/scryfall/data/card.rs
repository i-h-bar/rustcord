use serde::{Deserialize, Serialize};
use time::Date;
use time::serde::format_description;
use uuid::Uuid;

format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Serialize, Deserialize)]
pub struct ScryfallCard {
    pub id: Uuid,
    pub oracle_id: Uuid,
    pub name: String,
    #[serde(with = "date_format")]
    pub released_at: Date,
    #[serde(rename = "scryfall_uri")]
    pub url: String,
    
}
