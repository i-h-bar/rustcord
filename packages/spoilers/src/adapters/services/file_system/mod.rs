use std::env;
use std::hash::Hash;
use async_trait::async_trait;
use crate::ports::image_store::{Image, ImageStore};
use crate::ports::storage::{Card, CardInfo};

pub struct FileSystem {
    image_dir: String,
    illustration_dir: String,
}

impl FileSystem {
    pub fn new() -> Self {
        let base_dir = env::var("IMAGES_DIR").expect("Images dir wasn't in env vars");
        Self {
            image_dir: format!("{}/images/", &base_dir),
            illustration_dir: format!("{}/illustrations/", &base_dir),
        }
    }
}


#[async_trait]
impl ImageStore for FileSystem {
    async fn exists(&self, card: &CardInfo) -> bool {
        let path = format!("{}/{}.png", self.image_dir, card.image.id);

        tokio::fs::try_exists(&path).await.unwrap_or_else(|_| false)
    }

    async fn save(&self, image: Image) {
        let Image(id, bytes) = image;

        let path = format!("{}/{}.png", self.image_dir, id);
        
        if let Err(why) = tokio::fs::write(&path, bytes).await {
            log::error!("Failed to save file: {}", why);
        };
    }
}