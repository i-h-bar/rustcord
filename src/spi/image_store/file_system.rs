use crate::domain::card::Card;
use crate::spi::image_store::{ImageRetrievalError, ImageStore, Images};
use async_trait::async_trait;
use std::env;

pub struct FileSystem {
    image_dir: String,
    illustration_dir: String,
}

#[async_trait]
impl ImageStore for FileSystem {
    fn new() -> Self {
        let base_dir = env::var("IMAGES_DIR").expect("Images dir wasn't in env vars");
        Self {
            image_dir: format!("{}/images/", &base_dir),
            illustration_dir: format!("{}/illustrations/", &base_dir),
        }
    }

    async fn fetch(&self, card: &Card) -> Result<Images, ImageRetrievalError> {
        let (front_id, back_id) = card.image_ids();

        let front = tokio::fs::read(format!("{}{front_id}.png", self.image_dir))
            .await
            .map_err(|_| {
                ImageRetrievalError(format!("No front image found for {}", card.front_name))
            })?;

        let back = if let Some(back_id) = back_id {
            tokio::fs::read(format!("{}{back_id}.png", self.image_dir))
                .await
                .ok()
        } else {
            None
        };

        Ok(Images { front, back })
    }

    async fn fetch_illustration(&self, card: &Card) -> Result<Images, ImageRetrievalError> {
        let Some(illustration_id) = card.front_illustration_id() else {
            return Err(ImageRetrievalError(String::from(
                "Card had no illustration id",
            )));
        };

        let front = tokio::fs::read(format!("{}{}.png", self.illustration_dir, illustration_id,))
            .await
            .map_err(|_| {
                ImageRetrievalError(format!("No illustration found for {}", card.front_name))
            })?;

        Ok(Images { front, back: None })
    }
}
