use crate::ports::images::{ImageRetrievalError, ImageStore};
use async_trait::async_trait;
use contracts::card::Card;
use std::env;

pub struct FileSystem {
    image_dir: String,
}

impl FileSystem {
    #[must_use]
    pub fn from_base_dir(base_dir: &str) -> Self {
        Self {
            image_dir: format!("{base_dir}/images/"),
        }
    }
}

#[async_trait]
impl ImageStore for FileSystem {
    fn create() -> Self {
        let base_dir = env::var("IMAGES_DIR").expect("IMAGES_DIR wasn't in env vars");
        Self::from_base_dir(&base_dir)
    }

    async fn fetch(&self, card: &Card) -> Result<Vec<u8>, ImageRetrievalError> {
        tokio::fs::read(format!("{}{}", self.image_dir, card.image_file()))
            .await
            .map_err(|why| {
                log::warn!("Error getting image {why:?}");
                ImageRetrievalError::new(format!("No image found for {}", card.name()))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn test_card() -> Card {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        Card::new(
            id,
            "Lightning Bolt".to_string(),
            "lightning bolt".to_string(),
            id,
            "https://scryfall.com/card/x/1".to_string(),
            id,
            None,
            "{R}".to_string(),
            vec!["R".to_string()],
            None,
            None,
            None,
            None,
            "Instant".to_string(),
            "Deal damage".to_string(),
            None,
            "Artist".to_string(),
            "Alpha".to_string(),
            "LEA".to_string(),
            time::Date::from_calendar_date(1993, time::Month::August, 5).unwrap(),
        )
    }

    fn write_image(base: &std::path::Path, name: &str, bytes: &[u8]) {
        std::fs::create_dir_all(base.join("images")).unwrap();
        std::fs::write(base.join("images").join(name), bytes).unwrap();
    }

    #[tokio::test]
    async fn fetch_reads_webp_card_image() {
        let dir = tempfile::tempdir().unwrap();
        let card = test_card();
        write_image(dir.path(), &card.image_file().to_string(), b"webp");

        let store = FileSystem::from_base_dir(dir.path().to_str().unwrap());

        assert_eq!(store.fetch(&card).await.unwrap(), b"webp");
    }

    #[tokio::test]
    async fn fetch_ignores_legacy_png() {
        let dir = tempfile::tempdir().unwrap();
        let card = test_card();
        write_image(dir.path(), &format!("{}.png", card.image_id()), b"png");

        let store = FileSystem::from_base_dir(dir.path().to_str().unwrap());

        assert!(store.fetch(&card).await.is_err());
    }
}
