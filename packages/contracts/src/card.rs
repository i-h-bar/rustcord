use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Card {
    id: Uuid,
    name: String,
    normalised_name: String,
    oracle_id: Uuid,
    url: String,
    image_id: Uuid,
    illustration_id: Option<Uuid>,
    mana_cost: String,
    colour_identity: Vec<String>,
    power: Option<String>,
    toughness: Option<String>,
    loyalty: Option<String>,
    defence: Option<String>,
    type_line: String,
    oracle_text: String,
    back_id: Option<Uuid>,
    artist: String,
    set_name: String,
}

impl Card {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        name: String,
        normalised_name: String,
        oracle_id: Uuid,
        url: String,
        image_id: Uuid,
        illustration_id: Option<Uuid>,
        mana_cost: String,
        colour_identity: Vec<String>,
        power: Option<String>,
        toughness: Option<String>,
        loyalty: Option<String>,
        defence: Option<String>,
        type_line: String,
        oracle_text: String,
        back_id: Option<Uuid>,
        artist: String,
        set_name: String,
    ) -> Self {
        Self {
            id,
            name,
            normalised_name,
            oracle_id,
            url,
            image_id,
            illustration_id,
            mana_cost,
            colour_identity,
            power,
            toughness,
            loyalty,
            defence,
            type_line,
            oracle_text,
            back_id,
            artist,
            set_name,
        }
    }
    
    #[must_use]
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    
    #[must_use]
    pub fn normalised_name(&self) -> &str {
        &self.normalised_name
    }

    #[must_use]
    pub fn oracle_id(&self) -> &Uuid {
        &self.oracle_id
    }

    #[must_use]
    pub fn back_id(&self) -> Option<&Uuid> {
        self.back_id.as_ref()
    }

    #[must_use]
    pub fn image_id(&self) -> &Uuid {
        &self.image_id
    }

    #[must_use]
    pub fn illustration_id(&self) -> Option<&Uuid> {
        self.illustration_id.as_ref()
    }

    #[must_use]
    pub fn set_name(&self) -> &str {
        &self.set_name
    }
    
    #[must_use]
    pub fn toughness(&self) -> Option<&str> {
        self.toughness.as_deref()
    }
    
    #[must_use]
    pub fn loyalty(&self) -> Option<&str> {
        self.loyalty.as_deref()
    }
    
    #[must_use]
    pub fn defence(&self) -> Option<&str> {
        self.defence.as_deref()
    }
    
    #[must_use]
    pub fn type_line(&self) -> &str {
        &self.type_line
    }
    
    #[must_use]
    pub fn oracle_text(&self) -> &str {
        &self.oracle_text
    }
    
    #[must_use]
    pub fn power(&self) -> Option<&str> {
        self.power.as_deref()
    }
    
    #[must_use]
    pub fn artist(&self) -> &str {
        &self.artist
    }
    
    #[must_use]
    pub fn colour_identity(&self) -> &[String] {
        &self.colour_identity
    }
    
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }
    
    #[must_use]
    pub fn mana_cost(&self) -> &str {
        &self.mana_cost
    }
}

impl PartialEq<Card> for &str {
    fn eq(&self, other: &Card) -> bool {
        self == &other.normalised_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_card() -> Card {
        Card {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: String::from("Lightning Bolt"),
            normalised_name: String::from("lightning bolt"),
            url: String::from("https://scryfall.com/card/test/1"),
            image_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            oracle_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            illustration_id: Some(
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            ),
            mana_cost: String::from("{R}"),
            colour_identity: vec![String::from("R")],
            power: None,
            toughness: None,
            loyalty: None,
            defence: None,
            type_line: String::from("Instant"),
            oracle_text: String::from("Lightning Bolt deals 3 damage to any target."),
            back_id: None,
            artist: String::from("Christopher Rush"),
            set_name: String::from("Alpha"),
        }
    }

    fn create_double_faced_card() -> Card {
        let mut card = create_test_card();
        card.name = String::from("Delver of Secrets");
        card.normalised_name = String::from("delver of secrets");
        card.back_id = Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap());
        card
    }

    #[test]
    fn test_front_image_id() {
        let card = create_test_card();
        assert_eq!(
            card.image_id(),
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
            card.illustration_id(),
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap())
        );
    }

    #[test]
    fn test_front_illustration_id_none() {
        let mut card = create_test_card();
        card.illustration_id = None;
        assert!(card.illustration_id().is_none());
    }

    #[test]
    fn test_illustration_ids_single_face() {
        let card = create_test_card();
        let front = card.illustration_id();
        assert_eq!(
            front,
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap())
        );
    }

    #[test]
    fn test_illustration_ids_double_face() {
        let card = create_double_faced_card();
        let front = card.illustration_id();
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
        card2.name = String::from("Different Card");
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
