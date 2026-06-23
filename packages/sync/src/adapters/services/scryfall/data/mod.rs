pub mod card;
pub mod set;
pub mod symbols;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Object {
    List,
}

#[derive(Deserialize, Serialize)]
pub struct ScryfallData<T> {
    pub object: Object,
    pub has_more: bool,
    pub next_page: Option<String>,
    pub data: Vec<T>,
}
