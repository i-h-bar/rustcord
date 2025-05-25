use crate::mtg::card::FuzzyFound;
use once_cell::sync::OnceCell;
use serenity::all::CreateAttachment;
use std::env;
use std::sync::LazyLock;
use uuid::Uuid;

pub static IMAGE_FETCHER: LazyLock<ImageFetcher> = LazyLock::new(|| ImageFetcher::new());

#[derive(Debug)]
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
        fetch_image(&self.image_dir, card.image_ids()).await
    }

    pub async fn fetch_illustration(
        &self,
        card: &FuzzyFound,
    ) -> (Option<CreateAttachment>, Option<CreateAttachment>) {
        fetch_image(&self.illustration_dir, card.illustration_ids()).await
    }
}

async fn fetch_image(
    image_dir: &str,
    (front_id, back_id): (Option<&Uuid>, Option<&Uuid>),
) -> (Option<CreateAttachment>, Option<CreateAttachment>) {
    let front = if let Some(front_id) = front_id {
        let image = tokio::fs::read(format!("{}{}.png", image_dir, front_id))
            .await
            .ok();

        image.map(|image| CreateAttachment::bytes(image, format!("{}.png", front_id)))
    } else {
        None
    };

    let back = if let Some(back_id) = back_id {
        let image = tokio::fs::read(format!("{}{}.png", image_dir, &back_id))
            .await
            .ok();

        image.map(|image| CreateAttachment::bytes(image, format!("{}.png", back_id)))
    } else {
        None
    };

    (front, back)
}
