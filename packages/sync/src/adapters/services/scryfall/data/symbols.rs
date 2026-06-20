use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ScryfallSymbol {
    pub symbol: String,
    pub svg_uri: String,
}
