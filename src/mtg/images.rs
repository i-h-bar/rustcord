use crate::mtg::db::FuzzyFound;
use serenity::all::CreateAttachment;
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

    pub async fn fetch(
        &self,
        card: &FuzzyFound,
    ) -> (Option<CreateAttachment>, Option<CreateAttachment>) {
        let image = tokio::fs::read(format!("{}{}.png", &self.image_dir, &card.front_image_id))
            .await
            .ok();

        let front = if let Some(image) = image {
            Some(CreateAttachment::bytes(
                image,
                format!("{}.png", card.front_image_id),
            ))
        } else {
            None
        };

        let back = if let Some(back_image_id) = &card.back_image_id {
            let image = tokio::fs::read(format!("{}{}.png", &self.image_dir, &back_image_id))
                .await
                .ok();

            if let Some(image) = image {
                Some(CreateAttachment::bytes(
                    image,
                    format!("{}.png", back_image_id),
                ))
            } else {
                None
            }
        } else {
            None
        };

        (front, back)
    }
}
