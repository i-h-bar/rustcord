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