use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Serialize, Deserialize)]
pub struct ScryfallCard {
    pub id: Uuid,
    pub oracle_id: Uuid,
    pub name: String,
}