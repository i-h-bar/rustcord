use crate::ports::image_store::{Illustration, Image, ImageStore};
use crate::ports::source::CardSource;
use crate::ports::storage::CardInfo;
use futures::future;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "local-dev")]
use indicatif::{ProgressBar, ProgressStyle};

async fn download_and_save_image(
    id: Uuid,
    url: &str,
    image_store: &impl ImageStore,
    source: &impl CardSource,
) {
    if let Some(bytes) = source.download_image(url).await {
        image_store.save_image(Image(id, bytes)).await;
    }
}

async fn download_and_save_illustration(
    id: Uuid,
    url: &str,
    image_store: &impl ImageStore,
    source: &impl CardSource,
) {
    if let Some(bytes) = source.download_image(url).await {
        image_store.save_illustration(Illustration(id, bytes)).await;
    }
}

pub async fn save_images(
    changed_images: &HashMap<Uuid, String>,
    changed_illustrations: &HashMap<Uuid, String>,
    cards: &[CardInfo],
    image_store: &impl ImageStore,
    source: &impl CardSource,
) {
    log::info!(
        "Processing images: {} changed, {} cards to check",
        changed_images.len() + changed_illustrations.len(),
        cards.len()
    );

    #[cfg(feature = "local-dev")]
    let pb = {
        let total = changed_images.len() + changed_illustrations.len() + cards.len();
        let bar = ProgressBar::new(total as u64);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} images ({eta})",
            )
            .unwrap()
            .progress_chars("=>-"),
        );
        bar
    };

    future::join_all(changed_images.iter().map(|(id, url)| async {
        download_and_save_image(*id, url, image_store, source).await;
        #[cfg(feature = "local-dev")]
        pb.inc(1);
    }))
    .await;

    future::join_all(changed_illustrations.iter().map(|(id, url)| async {
        download_and_save_illustration(*id, url, image_store, source).await;
        #[cfg(feature = "local-dev")]
        pb.inc(1);
    }))
    .await;

    future::join_all(cards.iter().map(|card| async {
        if !changed_images.contains_key(&card.image.id)
            && !image_store.card_image_exists(card).await
        {
            download_and_save_image(card.image.id, &card.image.scryfall_url, image_store, source)
                .await;
        }

        if let Some(illustration) = &card.illustration
            && !changed_illustrations.contains_key(&illustration.id)
            && !image_store.card_illustration_exists(card).await
        {
            download_and_save_illustration(
                illustration.id,
                &illustration.scryfall_url,
                image_store,
                source,
            )
            .await;
        }

        #[cfg(feature = "local-dev")]
        pb.inc(1);
    }))
    .await;

    #[cfg(feature = "local-dev")]
    pb.finish_with_message("done");
}
