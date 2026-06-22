use crate::ports::image_store::{Illustration, Image, ImageStore};
use crate::ports::source::CardSource;
use crate::ports::storage::CardInfo;
use futures::future;

#[cfg(feature = "local-dev")]
use indicatif::{ProgressBar, ProgressStyle};

pub async fn save_images(
    cards: &[CardInfo],
    image_store: &impl ImageStore,
    source: &impl CardSource,
) {
    log::info!("Checking {} cards for missing images", cards.len());

    #[cfg(feature = "local-dev")]
    let pb = {
        let bar = ProgressBar::new(cards.len() as u64);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} images ({eta})",
            )
            .unwrap()
            .progress_chars("=>-"),
        );
        bar
    };

    future::join_all(cards.iter().map(|card| async {
        if !image_store.card_image_exists(card).await
            && let Some(bytes) = source.download_image(&card.image.scryfall_url).await
        {
            image_store.save_image(Image(card.image.id, bytes)).await;
        }

        if let Some(illustration) = &card.illustration
            && !image_store.card_illustration_exists(card).await
            && let Some(bytes) = source.download_image(&illustration.scryfall_url).await
        {
            image_store
                .save_illustration(Illustration(illustration.id, bytes))
                .await;
        }

        #[cfg(feature = "local-dev")]
        pb.inc(1);
    }))
    .await;

    #[cfg(feature = "local-dev")]
    pb.finish_with_message("done");
}
