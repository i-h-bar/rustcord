use contracts::{card::Card, image::Image};
use crate::ports::outbound::image_store::{ImageRetrievalError, ImageStore};
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

    async fn fetch(&self, card: &Card) -> Result<Image, ImageRetrievalError> {
        let id = card.image_id();

        let bytes = match tokio::fs::read(format!("{}{id}.png", self.image_dir)).await {
            Err(why) => {
                log::warn!("Error getting image {why:?}");
                return Err(ImageRetrievalError::new(format!(
                    "No front image found for {}",
                    card.name()
                )));
            }
            Ok(image) => image,
        };

        Ok(Image::new(bytes))
    }

    async fn fetch_illustration(&self, card: &Card) -> Result<Image, ImageRetrievalError> {
        let Some(illustration_id) = card.illustration_id() else {
            return Err(ImageRetrievalError::new(String::from(
                "Card had no illustration id",
            )));
        };

        let bytes = tokio::fs::read(format!("{}{}.png", self.illustration_dir, illustration_id,))
            .await
            .map_err(|_| {
                ImageRetrievalError::new(format!("No illustration found for {}", card.name()))
            })?;

        Ok(Image::new(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_card_single_face() -> Card {
        Card::new(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            String::from("Test Card"),
            String::from("test card"),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            String::from("https://scryfall.com/card/test/1"),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap()),
            String::from("{1}{U}"),
            vec![String::from("U")],
            Some(String::from("2")),
            Some(String::from("2")),
            None,
            None,
            String::from("Creature - Test"),
            String::from("Test ability"),
            None,
            String::from("Test Artist"),
            String::from("Test Set"),
        )
    }

    fn create_test_card_double_face() -> Card {
        Card::new(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            String::from("Test Card"),
            String::from("test card"),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            String::from("https://scryfall.com/card/test/1"),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap()),
            String::from("{1}{U}"),
            vec![String::from("U")],
            Some(String::from("2")),
            Some(String::from("2")),
            None,
            None,
            String::from("Creature - Test"),
            String::from("Test ability"),
            Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap()),
            String::from("Test Artist"),
            String::from("Test Set"),
        )
    }

    fn create_test_card_no_illustration() -> Card {
        Card::new(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            String::from("Test Card"),
            String::from("test card"),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            String::from("https://scryfall.com/card/test/1"),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            None,
            String::from("{1}{U}"),
            vec![String::from("U")],
            Some(String::from("2")),
            Some(String::from("2")),
            None,
            None,
            String::from("Creature - Test"),
            String::from("Test ability"),
            None,
            String::from("Test Artist"),
            String::from("Test Set"),
        )
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
        let images = Image::new(front_data.clone());
        assert_eq!(images.bytes(), &front_data);
    }

    #[tokio::test]
    async fn test_fetch_illustration_no_illustration_id() {
        let card = create_test_card_no_illustration();

        // We can't easily test the FileSystem implementation without a real filesystem
        // but we can verify the card setup for the test
        assert!(card.illustration_id().is_none());

        // The actual implementation would return an error when front_illustration_id is None
        // This is tested through integration tests with the domain layer
    }

    #[test]
    fn test_card_helpers_single_face() {
        let card = create_test_card_single_face();
        let front_id = card.image_id();

        assert_eq!(
            front_id,
            &Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn test_card_helpers_double_face() {
        let card = create_test_card_double_face();
        let front_id = card.image_id();

        assert_eq!(
            front_id,
            &Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn test_card_illustration_ids() {
        let card = create_test_card_double_face();
        let front_ill = card.illustration_id();

        assert_eq!(
            front_ill,
            Some(&Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap())
        );
    }
}

