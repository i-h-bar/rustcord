use crate::domain::dto::search_result::SearchResultDto;
use crate::ports::inbound::client::MessageInteraction;

pub async fn card_response<MI: MessageInteraction>(
    result: Option<SearchResultDto>,
    interaction: &MI,
) {
    match result {
        None => {
            if let Err(why) = interaction
                .reply(String::from("Failed to find card :("))
                .await
            {
                log::error!("Error sending card not found message :( {why:?}");
            }
        }
        Some(result) => {
            if let Err(why) = interaction.send_card(result).await {
                log::error!("Error sending card message :( {why:?}");
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dto::card::Card;
    use crate::domain::dto::search_result::SearchResultDto;
    use crate::ports::inbound::client::{MessageInteractionError, MockMessageInteraction};
    use crate::ports::outbound::image_store::Image;
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

    #[tokio::test]
    async fn test_card_response_with_card_success() {
        let card = create_test_card();
        let images = Image::new(vec![1, 2, 3]);
        let result = Some(SearchResultDto::new(card.clone(), images.clone()));

        let mut mock_interaction = MockMessageInteraction::new();
        mock_interaction
            .expect_send_card()
            .withf(move |r| r.card() == &card && r.image().bytes() == images.bytes())
            .times(1)
            .returning(|_| Ok(()));

        card_response(result, &mock_interaction).await;
    }

    #[tokio::test]
    async fn test_card_response_with_card_error() {
        let card = create_test_card();
        let images = Image::new(vec![1, 2, 3]);
        let result = Some(SearchResultDto::new(card, images));

        let mut mock_interaction = MockMessageInteraction::new();
        mock_interaction
            .expect_send_card()
            .times(1)
            .returning(|_| Err(MessageInteractionError::new(String::from("Send error"))));

        // Should not panic even when send_card fails
        card_response(result, &mock_interaction).await;
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
