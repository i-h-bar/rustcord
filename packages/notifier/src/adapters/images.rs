use crate::ports::images::{ImageRetrievalError, ImageStore};
use async_trait::async_trait;
use contracts::card::Card;
use std::env;

pub struct FileSystem {
    image_dir: String,
}

#[async_trait]
impl ImageStore for FileSystem {
    fn create() -> Self {
        let base_dir = env::var("IMAGES_DIR").expect("IMAGES_DIR wasn't in env vars");
        Self {
            image_dir: format!("{base_dir}/images/"),
        }
    }

    async fn fetch(&self, card: &Card) -> Result<Vec<u8>, ImageRetrievalError> {
        let id = card.image_id();

        tokio::fs::read(format!("{}{id}.png", self.image_dir))
            .await
            .map_err(|why| {
                log::warn!("Error getting image {why:?}");
                ImageRetrievalError::new(format!("No image found for {}", card.name()))
            })
    }
}
