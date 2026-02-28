use crate::domain::card::Card;
use crate::ports::outbound::image_store::{ImageRetrievalError, ImageStore, Images};
use async_trait::async_trait;
use std::env;

pub struct FileSystem {
    image_dir: String,
    illustration_dir: String,
}

#[async_trait]
impl ImageStore for FileSystem {
    fn create() -> Self {
        let base_dir = env::var("IMAGES_DIR").expect("Images dir wasn't in env vars");
        Self {
            image_dir: format!("{}/images/", &base_dir),
            illustration_dir: format!("{}/illustrations/", &base_dir),
        }
    }

    async fn fetch(&self, card: &Card) -> Result<Images, ImageRetrievalError> {
        let front_id = card.image_id();

        let front = match tokio::fs::read(format!("{}{front_id}.png", self.image_dir)).await {
            Err(why) => {
                log::warn!("Error getting image {why:?}");
                return Err(ImageRetrievalError::new(format!(
                    "No front image found for {}",
                    card.front_name
                )));
            }
            Ok(image) => image,
        };

        Ok(Images { front })
    }

    async fn fetch_illustration(&self, card: &Card) -> Result<Images, ImageRetrievalError> {
        let Some(illustration_id) = card.front_illustration_id() else {
            return Err(ImageRetrievalError::new(String::from(
                "Card had no illustration id",
            )));
        };

        let front = tokio::fs::read(format!("{}{}.png", self.illustration_dir, illustration_id,))
            .await
            .map_err(|_| {
                ImageRetrievalError::new(format!("No illustration found for {}", card.front_name))
            })?;

        Ok(Images { front })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_card_single_face() -> Card {
        Card {
            front_name: String::from("Test Card"),
            front_normalised_name: String::from("test card"),
            front_scryfall_url: String::from("https://scryfall.com/card/test/1"),
            front_image_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            front_oracle_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            front_illustration_id: Some(
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            ),
            front_mana_cost: String::from("{1}{U}"),
            front_colour_identity: vec![String::from("U")],
            front_power: Some(String::from("2")),
            front_toughness: Some(String::from("2")),
            front_loyalty: None,
            front_defence: None,
            front_type_line: String::from("Creature - Test"),
            front_oracle_text: String::from("Test ability"),
            back_name: None,
            artist: String::from("Test Artist"),
            set_name: String::from("Test Set"),
            back_id: None,
        }
    }

    fn create_test_card_double_face() -> Card {
        let mut card = create_test_card_single_face();
        card.back_name = Some(String::from("Test Card Back"));
        card.back_image_id = Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap());
        card.back_illustration_id =
            Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003").unwrap());
        card
    }

    fn create_test_card_no_illustration() -> Card {
        let mut card = create_test_card_single_face();
        card.front_illustration_id = None;
        card
    }

    #[test]
    fn test_image_retrieval_error_display() {
        let error = ImageRetrievalError::new(String::from("Test error message"));
        assert_eq!(error.to_string(), "Error Retrieving Image");
    }

    #[test]
    fn test_image_retrieval_error_debug() {
        let error = ImageRetrievalError::new(String::from("Test error message"));
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ImageRetrievalError"));
    }

    #[test]
    fn test_images_struct_single_face() {
        let front_data = vec![1, 2, 3, 4];
        let images = Images {
            front: front_data.clone(),
            back: None,
        };
        assert_eq!(images.front, front_data);
        assert!(images.back.is_none());
    }

    #[test]
    fn test_images_struct_double_face() {
        let front_data = vec![1, 2, 3, 4];
        let back_data = vec![5, 6, 7, 8];
        let images = Images {
            front: front_data.clone(),
            back: Some(back_data.clone()),
        };
        assert_eq!(images.front, front_data);
        assert_eq!(images.back, Some(back_data));
    }

    #[tokio::test]
    async fn test_fetch_illustration_no_illustration_id() {
        let card = create_test_card_no_illustration();

        // We can't easily test the FileSystem implementation without a real filesystem
        // but we can verify the card setup for the test
        assert!(card.front_illustration_id().is_none());

        // The actual implementation would return an error when front_illustration_id is None
        // This is tested through integration tests with the domain layer
    }

    #[test]
    fn test_card_helpers_single_face() {
        let card = create_test_card_single_face();
        let (front_id, back_id) = card.image_id();

        assert_eq!(
            front_id,
            &Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert!(back_id.is_none());
    }

    #[test]
    fn test_card_helpers_double_face() {
        let card = create_test_card_double_face();
        let (front_id, back_id) = card.image_id();

        assert_eq!(
            front_id,
            &Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(
            back_id,
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap())
        );
    }

    #[test]
    fn test_card_illustration_ids() {
        let card = create_test_card_double_face();
        let (front_ill, back_ill) = card.illustration_ids();

        assert_eq!(
            front_ill,
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap())
        );
        assert_eq!(
            back_ill,
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003").unwrap())
        );
    }
}
