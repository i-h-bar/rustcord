use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::utils::fuzzy::ToBytes;

#[cfg_attr(test, derive(Clone, PartialEq))]
#[derive(Debug, Deserialize, Serialize)]
pub struct Card {
    pub front_name: String,
    pub front_normalised_name: String,
    pub front_oracle_id: Uuid,
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
    pub back_id: Option<Uuid>,
    pub artist: String,
    pub set_name: String,
}

impl Card {
    #[must_use]
    pub fn front_oracle_id(&self) -> &Uuid {
        &self.front_oracle_id
    }

    #[must_use]
    pub fn back_id(&self) -> Option<&Uuid> {
        self.back_id.as_ref()
    }

    #[must_use]
    pub fn image_id(&self) -> &Uuid {
        &self.front_image_id
    }

    #[must_use]
    pub fn front_image_id(&self) -> &Uuid {
        &self.front_image_id
    }

    #[must_use]
    pub fn front_illustration_id(&self) -> Option<&Uuid> {
        self.front_illustration_id.as_ref()
    }

    #[must_use]
    pub fn illustration_ids(&self) -> Option<&Uuid> {
        self.front_illustration_id.as_ref()
    }

    #[must_use]
    pub fn set_name(&self) -> &str {
        &self.set_name
    }
}

impl ToBytes for Card {
    fn to_bytes(&self) -> &[u8] {
        self.front_name.as_bytes()
    }
}

impl PartialEq<Card> for &str {
    fn eq(&self, other: &Card) -> bool {
        self == &other.front_normalised_name
    }
}