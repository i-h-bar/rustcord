use crate::ports::drivers::client::MessageInteraction;
use contracts::search_result::SearchResultDto;

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
    use crate::ports::drivers::client::{MessageInteractionError, MockMessageInteraction};
    use contracts::card::Card;
    use contracts::image::Image;
    use contracts::search_result::SearchResultDto;
    use uuid::Uuid;

    fn create_test_card() -> Card {
        Card::new(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            String::from("Lightning Bolt"),
            String::from("lightning bolt"),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            String::from("https://scryfall.com/card/test/1"),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap()),
            String::from("{R}"),
            vec![String::from("R")],
            None,
            None,
            None,
            None,
            String::from("Instant"),
            String::from("Lightning Bolt deals 3 damage to any target."),
            None,
            String::from("Christopher Rush"),
            String::from("Alpha"),
        )
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
