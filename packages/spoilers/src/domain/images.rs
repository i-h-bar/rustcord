use crate::ports::image_store::ImageStore;
use crate::ports::source::CardSource;
use crate::ports::storage::CardInfo;
use futures::future;
use std::sync::Arc;
use tokio::sync::Semaphore;

async fn save_card(
    card: &CardInfo,
    image_store: &impl ImageStore,
    source: &impl CardSource,
    sem: Arc<Semaphore>,
) {
    if !image_store.card_image_exists(card).await {
        let image = {
            let Ok(_guard) = sem.acquire().await else {
                log::error!("Poisoned semaphore");
                return;
            };
            source.get_image(card).await
        };

        if let Some(image) = image {
            image_store.save_image(image).await;
        }
    }

    if !image_store.card_illustration_exists(card).await {
        let illustration = {
            let Ok(_guard) = sem.acquire().await else {
                log::error!("Poisoned semaphore");
                return;
            };
            source.get_illustration(card).await
        };
        if let Some(illustration) = illustration {
            image_store.save_illustration(illustration).await;
        }
    }
}

pub async fn save_images(
    cards: &[CardInfo],
    image_store: &impl ImageStore,
    source: &impl CardSource,
) {
    log::info!("Saving {} images", cards.len());

    let sem = Arc::new(Semaphore::new(5));

    future::join_all(
        cards
            .iter()
            .map(|card| save_card(card, image_store, source, Arc::clone(&sem))),
    )
    .await;
}
