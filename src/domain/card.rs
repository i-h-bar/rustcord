use crate::domain::search::CardAndImage;
use crate::domain::utils::fuzzy::ToBytes;
use crate::ports::inbound::client::MessageInteraction;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(test, derive(Clone, PartialEq))]
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

pub async fn card_response<MI: MessageInteraction>(card: Option<CardAndImage>, interaction: &MI) {
    match card {
        None => {
            if let Err(why) = interaction
                .reply(String::from("Failed to find card :("))
                .await
            {
                log::error!("Error sending card not found message :( {why:?}");
            }
        }
        Some((card, images)) => {
            if let Err(why) = interaction.send_card(card, images).await {
                log::error!("Error sending card message :( {why:?}");
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::inbound::client::{MessageInteractionError, MockMessageInteraction};
    use crate::ports::outbound::image_store::Images;

    fn create_test_card() -> Card {
        Card {
            front_name: String::from("Lightning Bolt"),
            front_normalised_name: String::from("lightning bolt"),
            front_scryfall_url: String::from("https://scryfall.com/card/test/1"),
            front_image_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
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
            back_name: None,
            back_scryfall_url: None,
            back_image_id: None,
            back_illustration_id: None,
            back_mana_cost: None,
            back_colour_identity: None,
            back_power: None,
            back_toughness: None,
            back_loyalty: None,
            back_defence: None,
            back_type_line: None,
            back_oracle_text: None,
            artist: String::from("Christopher Rush"),
            set_name: String::from("Alpha"),
        }
    }

    fn create_double_faced_card() -> Card {
        let mut card = create_test_card();
        card.front_name = String::from("Delver of Secrets");
        card.front_normalised_name = String::from("delver of secrets");
        card.back_name = Some(String::from("Insectile Aberration"));
        card.back_image_id = Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap());
        card.back_illustration_id =
            Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003").unwrap());
        card.back_scryfall_url = Some(String::from("https://scryfall.com/card/test/1/back"));
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
        assert!(card.back_image_id().is_none());
    }

    #[test]
    fn test_back_image_id_some() {
        let card = create_double_faced_card();
        assert_eq!(
            card.back_image_id(),
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap())
        );
    }

    #[test]
    fn test_image_ids_single_face() {
        let card = create_test_card();
        let (front, back) = card.image_ids();
        assert_eq!(
            front,
            &Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert!(back.is_none());
    }

    #[test]
    fn test_image_ids_double_face() {
        let card = create_double_faced_card();
        let (front, back) = card.image_ids();
        assert_eq!(
            front,
            &Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(
            back,
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap())
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
        let (front, back) = card.illustration_ids();
        assert_eq!(
            front,
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap())
        );
        assert!(back.is_none());
    }

    #[test]
    fn test_illustration_ids_double_face() {
        let card = create_double_faced_card();
        let (front, back) = card.illustration_ids();
        assert_eq!(
            front,
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap())
        );
        assert_eq!(
            back,
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003").unwrap())
        );
    }

    #[test]
    fn test_set_name() {
        let card = create_test_card();
        assert_eq!(card.set_name(), "Alpha");
    }

    #[test]
    fn test_to_bytes() {
        let card = create_test_card();
        assert_eq!(card.to_bytes(), "Lightning Bolt".as_bytes());
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

    #[tokio::test]
    async fn test_card_response_with_card_success() {
        let card = create_test_card();
        let images = Images {
            front: vec![1, 2, 3],
            back: None,
        };
        let card_and_image = Some((card.clone(), images.clone()));

        let mut mock_interaction = MockMessageInteraction::new();
        mock_interaction
            .expect_send_card()
            .withf(move |c, i| c == &card && i.front == images.front && i.back == images.back)
            .times(1)
            .returning(|_, _| Ok(()));

        card_response(card_and_image, &mock_interaction).await;
    }

    #[tokio::test]
    async fn test_card_response_with_card_error() {
        let card = create_test_card();
        let images = Images {
            front: vec![1, 2, 3],
            back: None,
        };
        let card_and_image = Some((card, images));

        let mut mock_interaction = MockMessageInteraction::new();
        mock_interaction
            .expect_send_card()
            .times(1)
            .returning(|_, _| Err(MessageInteractionError::new(String::from("Send error"))));

        // Should not panic even when send_card fails
        card_response(card_and_image, &mock_interaction).await;
    }

    #[tokio::test]
    async fn test_card_response_none_success() {
        let mut mock_interaction = MockMessageInteraction::new();
        mock_interaction
            .expect_reply()
            .with(mockall::predicate::eq(String::from(
                "Failed to find card :(",
            )))
            .times(1)
            .returning(|_| Ok(()));

        card_response(None, &mock_interaction).await;
    }

    #[tokio::test]
    async fn test_card_response_none_error() {
        let mut mock_interaction = MockMessageInteraction::new();
        mock_interaction
            .expect_reply()
            .with(mockall::predicate::eq(String::from(
                "Failed to find card :(",
            )))
            .times(1)
            .returning(|_| Err(MessageInteractionError::new(String::from("Reply error"))));

        // Should not panic even when reply fails
        card_response(None, &mock_interaction).await;
    }
}
