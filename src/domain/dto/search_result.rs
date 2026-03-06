use crate::domain::dto::card::Card;
use crate::domain::set::Set;
use crate::ports::outbound::image_store::Images;

pub struct SearchResultDto {
    card: Card,
    image: Images,
    printings: Option<Vec<Set>>,
    similar_cards: Option<Vec<Card>>,
}

impl SearchResultDto {
    #[must_use]
    pub fn new(card: Card, image: Images) -> Self {
        Self {
            card,
            image,
            printings: None,
            similar_cards: None,
        }
    }

    #[must_use]
    pub fn add_printings(mut self, set: Option<Vec<Set>>) -> Self {
        self.printings = set;

        self
    }

    #[must_use]
    pub fn add_similar_cards(mut self, set: Vec<Card>) -> Self {
        self.similar_cards = Some(set);
        self
    }

    #[must_use]
    pub fn image(&self) -> &Images {
        &self.image
    }

    #[must_use]
    pub fn card(&self) -> &Card {
        &self.card
    }

    #[must_use]
    pub fn printings(&self) -> Option<&Vec<Set>> {
        self.printings.as_ref()
    }

    #[must_use]
    pub fn similar_cards(&self) -> Option<&Vec<Card>> {
        self.similar_cards.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_card() -> Card {
        Card {
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

    #[test]
    fn test_add_printings() {
        let image = Images { front: vec![] };
        let card = create_test_card();
        let similar_cards = vec![create_test_card()];
        let printings = vec![Set::new(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            String::from("Alpha"),
        )];

        let result = SearchResultDto::new(card, image)
            .add_printings(Some(printings))
            .add_similar_cards(similar_cards);

        assert!(result.similar_cards.is_some());
        let similar_cards = result.similar_cards.unwrap();
        assert_eq!(similar_cards.len(), 1);
        assert_eq!(similar_cards[0].front_name, "Lightning Bolt".to_string());

        assert!(result.printings.is_some());
        let printings = result.printings.unwrap();
        assert_eq!(printings.len(), 1);
        assert_eq!(
            printings[0].card_id().to_string(),
            "550e8400-e29b-41d4-a716-446655440000".to_string()
        );
        assert_eq!(printings[0].name().to_string(), "Alpha".to_string());
    }
}
