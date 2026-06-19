use crate::ports::image_store::ImageStore;
use crate::ports::source::CardSource;
use crate::ports::storage::CardInfo;
use futures::future;

#[cfg(feature = "local-dev")]
use indicatif::{ProgressBar, ProgressStyle};

async fn save_card(card: &CardInfo, image_store: &impl ImageStore, source: &impl CardSource) {
    if !image_store.card_image_exists(card).await
        && let Some(image) = source.get_image(card).await
    {
        image_store.save_image(image).await;
    }

    if !image_store.card_illustration_exists(card).await
        && let Some(illustration) = source.get_illustration(card).await
    {
        image_store.save_illustration(illustration).await;
    }
}

pub async fn save_images(
    cards: &[CardInfo],
    image_store: &impl ImageStore,
    source: &impl CardSource,
) {
    log::info!("Saving {} images", cards.len());

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
        save_card(card, image_store, source).await;
        #[cfg(feature = "local-dev")]
        pb.inc(1);
    }))
    .await;

    #[cfg(feature = "local-dev")]
    pb.finish_with_message("done");
}
