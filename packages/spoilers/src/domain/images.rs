use std::sync::Arc;
use futures::future;
use tokio::sync::Semaphore;
use crate::ports::image_store::ImageStore;
use crate::ports::source::CardSource;
use crate::ports::storage::CardInfo;

async fn save_image(card: &CardInfo, image_store: &impl ImageStore, source: &impl CardSource, sem: Arc<Semaphore>) {
    if image_store.exists(card).await {
        log::debug!("Image already exists for {}", card.card.name);
        return;
    }

    let image = {
        let _ = sem.acquire().await.unwrap();
        source.get_image(card).await
    };

    image_store.save(image).await;
}

pub async fn save_images(cards: &[CardInfo], image_store: &impl ImageStore, source: &impl CardSource) {
    log::info!("Saving {} images", cards.len());

    let sem = Arc::new(Semaphore::new(5));

    future::join_all(
        cards.iter().map(|card| save_image(card, image_store, source, Arc::clone(&sem))),
    ).await;
}