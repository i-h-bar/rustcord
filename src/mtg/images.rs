use crate::mtg::db::FuzzyFound;
use std::env;

pub struct ImageFetcher {
    image_dir: String,
    illustration_dir: String,
}

impl ImageFetcher {
    pub fn new() -> Self {
        let base_dir = env::var("IMAGES_DIR").expect("Images dir wasn't in env vars");
        Self {
            image_dir: format!("{}/images/", &base_dir),
            illustration_dir: format!("{}/illustrations/", &base_dir),
        }
    }

    pub async fn fetch(&self, card: &FuzzyFound) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
        let front = tokio::fs::read(format!("{}{}.png", &self.image_dir, &card.front_image_id))
            .await
            .ok();

        let back = if let Some(back_image_id) = &card.back_image_id {
            tokio::fs::read(format!("{}{}.png", &self.image_dir, &back_image_id))
                .await
                .ok()
        } else {
            None
        };

        (front, back)
    }
}
