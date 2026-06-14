pub mod card;

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
    pub data: Vec<T>,
}
