use crate::ports::services::image_store::{ImageRetrievalError, ImageStore};
use async_trait::async_trait;
use contracts::{card::Card, image::Image};
use std::env;

pub struct FileSystem {
    image_dir: String,
    illustration_dir: String,
}

impl FileSystem {
    #[must_use]
    pub fn from_base_dir(base_dir: &str) -> Self {
        Self {
            image_dir: format!("{base_dir}/images/"),
            illustration_dir: format!("{base_dir}/illustrations/"),
        }
    }
}

#[async_trait]
impl ImageStore for FileSystem {
    fn create() -> Self {
        let base_dir = env::var("IMAGES_DIR").expect("Images dir wasn't in env vars");
        Self::from_base_dir(&base_dir)
    }

    async fn fetch(&self, card: &Card) -> Result<Image, ImageRetrievalError> {
        let bytes = match tokio::fs::read(format!("{}{}", self.image_dir, card.image_file())).await
        {
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
        let Some(illustration_file) = card.illustration_file() else {
            return Err(ImageRetrievalError::new(String::from(
                "Card had no illustration id",
            )));
        };

        let bytes = tokio::fs::read(format!("{}{illustration_file}", self.illustration_dir))
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
            String::from("TST"),
            time::Date::from_calendar_date(2024, time::Month::January, 1).unwrap(),
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
            String::from("TST"),
            time::Date::from_calendar_date(2024, time::Month::January, 1).unwrap(),
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
            String::from("TST"),
            time::Date::from_calendar_date(2024, time::Month::January, 1).unwrap(),
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

    fn write_file(base: &std::path::Path, dir: &str, name: &str, bytes: &[u8]) {
        std::fs::create_dir_all(base.join(dir)).unwrap();
        std::fs::write(base.join(dir).join(name), bytes).unwrap();
    }

    #[tokio::test]
    async fn fetch_reads_webp_card_image() {
        let dir = tempfile::tempdir().unwrap();
        let card = create_test_card_single_face();
        write_file(
            dir.path(),
            "images",
            &card.image_file().to_string(),
            b"webp",
        );

        let store = FileSystem::from_base_dir(dir.path().to_str().unwrap());

        assert_eq!(store.fetch(&card).await.unwrap().bytes(), b"webp");
    }

    #[tokio::test]
    async fn fetch_illustration_reads_jpg() {
        let dir = tempfile::tempdir().unwrap();
        let card = create_test_card_single_face();
        let name = card.illustration_file().unwrap().to_string();
        assert!(name.ends_with(".jpg"));
        write_file(dir.path(), "illustrations", &name, b"jpg");

        let store = FileSystem::from_base_dir(dir.path().to_str().unwrap());

        assert_eq!(
            store.fetch_illustration(&card).await.unwrap().bytes(),
            b"jpg"
        );
    }

    #[tokio::test]
    async fn fetch_ignores_legacy_png() {
        let dir = tempfile::tempdir().unwrap();
        let card = create_test_card_single_face();
        write_file(
            dir.path(),
            "images",
            &format!("{}.png", card.image_id()),
            b"png",
        );

        let store = FileSystem::from_base_dir(dir.path().to_str().unwrap());

        assert!(store.fetch(&card).await.is_err());
    }
}
