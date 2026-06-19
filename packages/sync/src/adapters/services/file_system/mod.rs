use crate::ports::image_store::{Illustration, Image, ImageStore};
use crate::ports::storage::CardInfo;
use async_trait::async_trait;
use std::env;

pub struct FileSystem {
    image_dir: String,
    illustration_dir: String,
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem {
    /// # Panics
    /// Panics if the `IMAGES_DIR` environment variable is not set.
    #[must_use]
    pub fn new() -> Self {
        let base_dir = env::var("IMAGES_DIR").expect("Images dir wasn't in env vars");
        Self {
            image_dir: format!("{}/images/", &base_dir),
            illustration_dir: format!("{}/illustrations/", &base_dir),
        }
    }

    pub async fn save(&self, path: &str, bytes: Vec<u8>) {
        if let Err(why) = tokio::fs::write(&path, bytes).await {
            log::error!("Failed to save file: {why}");
        }
    }
}

#[async_trait]
impl ImageStore for FileSystem {
    async fn card_image_exists(&self, card: &CardInfo) -> bool {
        let path = format!("{}{}.png", self.image_dir, card.image.id);

        tokio::fs::try_exists(&path).await.unwrap_or(false)
    }

    async fn card_illustration_exists(&self, card: &CardInfo) -> bool {
        let Some(illustration) = card.illustration.as_ref() else {
            return true;
        };

        let path = format!("{}{}.png", self.illustration_dir, illustration.id);

        tokio::fs::try_exists(&path).await.unwrap_or(false)
    }

    async fn save_image(&self, image: Image) {
        let Image(id, bytes) = image;

        let path = format!("{}{}.png", self.image_dir, id);

        self.save(&path, bytes).await;
    }

    async fn save_illustration(&self, illustration: Illustration) {
        let Illustration(id, bytes) = illustration;

        let path = format!("{}{}.png", self.illustration_dir, id);

        self.save(&path, bytes).await;
    }
}
