use crate::domain::search::CardAndImage;
use crate::domain::utils::fuzzy::ToChars;
use crate::ports::clients::MessageInteraction;
use serde::{Deserialize, Serialize};
use std::str::Chars;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Card {
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

impl Card {
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

    #[must_use]
    pub fn set_name(&self) -> &str {
        &self.set_name
    }
}

impl ToChars for Card {
    fn to_chars(&self) -> Chars<'_> {
        self.front_normalised_name.chars()
    }
}

impl PartialEq<Card> for &str {
    fn eq(&self, other: &Card) -> bool {
        self == &other.front_normalised_name
    }
}

pub async fn card_response<MI: MessageInteraction>(card: Option<CardAndImage>, interaction: &MI) {
    match card {
        None => {
            if let Err(why) = interaction
                .reply(String::from("Failed to find card :("))
                .await
            {
                log::error!("Error sending card not found message :( {:?}", why);
            }
        }
        Some((card, images)) => {
            if let Err(why) = interaction.send_card(card, images).await {
                log::error!("Error sending card message :( {:?}", why);
            };
        }
    }
}
