use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(test, derive(Clone, PartialEq))]
#[derive(Debug, Deserialize, Serialize)]
pub struct Card {
    id: Uuid,
    front_name: String,
    front_normalised_name: String,
    front_oracle_id: Uuid,
    front_scryfall_url: String,
    front_image_id: Uuid,
    front_illustration_id: Option<Uuid>,
    front_mana_cost: String,
    front_colour_identity: Vec<String>,
    front_power: Option<String>,
    front_toughness: Option<String>,
    front_loyalty: Option<String>,
    front_defence: Option<String>,
    front_type_line: String,
    front_oracle_text: String,
    back_id: Option<Uuid>,
    artist: String,
    set_name: String,
}

impl Card {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        front_name: String,
        front_normalised_name: String,
        front_oracle_id: Uuid,
        front_scryfall_url: String,
        front_image_id: Uuid,
        front_illustration_id: Option<Uuid>,
        front_mana_cost: String,
        front_colour_identity: Vec<String>,
        front_power: Option<String>,
        front_toughness: Option<String>,
        front_loyalty: Option<String>,
        front_defence: Option<String>,
        front_type_line: String,
        front_oracle_text: String,
        back_id: Option<Uuid>,
        artist: String,
        set_name: String,
    ) -> Self {
        Self {
            id,
            front_name,
            front_normalised_name,
            front_oracle_id,
            front_scryfall_url,
            front_image_id,
            front_illustration_id,
            front_mana_cost,
            front_colour_identity,
            front_power,
            front_toughness,
            front_loyalty,
            front_defence,
            front_type_line,
            front_oracle_text,
            back_id,
            artist,
            set_name,
        }
    }
    
    #[must_use]
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.front_name
    }

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

impl PartialEq<Card> for &str {
    fn eq(&self, other: &Card) -> bool {
        self == &other.front_normalised_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_card() -> Card {
        Card {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            front_name: String::from("Lightning Bolt"),
            front_normalised_name: String::from("lightning bolt"),
            front_scryfall_url: String::from("https://scryfall.com/card/test/1"),
            front_image_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            front_oracle_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            front_illustration_id: Some(
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            ),
            front_mana_cost: String::from("{R}"),
            front_colour_identity: vec![String::from("R")],
            front_power: None,
            front_toughness: None,
            front_loyalty: None,
            front_defence: None,
            front_type_line: String::from("Instant"),
            front_oracle_text: String::from("Lightning Bolt deals 3 damage to any target."),
            back_id: None,
            artist: String::from("Christopher Rush"),
            set_name: String::from("Alpha"),
        }
    }

    fn create_double_faced_card() -> Card {
        let mut card = create_test_card();
        card.front_name = String::from("Delver of Secrets");
        card.front_normalised_name = String::from("delver of secrets");
        card.back_id = Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap());
        card
    }

    #[test]
    fn test_front_image_id() {
        let card = create_test_card();
        assert_eq!(
            card.front_image_id(),
            &Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn test_back_image_id_none() {
        let card = create_test_card();
        assert!(card.back_id().is_none());
    }

    #[test]
    fn test_back_image_id_some() {
        let card = create_double_faced_card();
        assert_eq!(
            card.back_id(),
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap())
        );
    }

    #[test]
    fn test_image_ids_single_face() {
        let card = create_test_card();
        let front = card.image_id();
        assert_eq!(
            front,
            &Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn test_image_ids_double_face() {
        let card = create_double_faced_card();
        let front = card.image_id();
        assert_eq!(
            front,
            &Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn test_front_illustration_id_some() {
        let card = create_test_card();
        assert_eq!(
            card.front_illustration_id(),
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap())
        );
    }

    #[test]
    fn test_front_illustration_id_none() {
        let mut card = create_test_card();
        card.front_illustration_id = None;
        assert!(card.front_illustration_id().is_none());
    }

    #[test]
    fn test_illustration_ids_single_face() {
        let card = create_test_card();
        let front = card.illustration_ids();
        assert_eq!(
            front,
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap())
        );
    }

    #[test]
    fn test_illustration_ids_double_face() {
        let card = create_double_faced_card();
        let front = card.illustration_ids();
        assert_eq!(
            front,
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap())
        );
    }

    #[test]
    fn test_set_name() {
        let card = create_test_card();
        assert_eq!(card.set_name(), "Alpha");
    }

    #[test]
    fn test_card_clone() {
        let card = create_test_card();
        let cloned = card.clone();
        assert_eq!(card, cloned);
    }

    #[test]
    fn test_card_partial_eq() {
        let card1 = create_test_card();
        let card2 = create_test_card();
        assert_eq!(card1, card2);
    }

    #[test]
    fn test_card_not_equal() {
        let card1 = create_test_card();
        let mut card2 = create_test_card();
        card2.front_name = String::from("Different Card");
        assert_ne!(card1, card2);
    }

    #[test]
    fn test_card_debug() {
        let card = create_test_card();
        let debug_str = format!("{:?}", card);
        assert!(debug_str.contains("Lightning Bolt"));
        assert!(debug_str.contains("lightning bolt"));
    }
}
