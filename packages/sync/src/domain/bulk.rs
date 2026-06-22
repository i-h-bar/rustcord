use crate::domain::utils::emoji;
use crate::domain::utils::images::save_images;
use crate::ports::emoji::EmojiStore;
use crate::ports::image_store::ImageStore;
use crate::ports::source::CardSource;
use crate::ports::storage::Storage;

pub async fn sync(
    source: impl CardSource,
    storage: impl Storage,
    image_store: impl ImageStore,
    emoji_store: impl EmojiStore,
) {
    source.get_all_sets().await;
    emoji::sync(&source, &emoji_store).await;

    let cards = source.fetch_all_cards().await;
    if cards.is_empty() {
        log::info!("No cards fetched");
        return;
    }

    let upsert_result = storage.upsert_cards(&cards).await;

    save_images(&cards, &image_store, &source).await;

    let deleted_images = storage
        .delete_orphaned_images(&upsert_result.orphaned_images)
        .await;
    let deleted_illustrations = storage
        .delete_orphaned_illustrations(&upsert_result.orphaned_illustrations)
        .await;

    for id in deleted_images {
        image_store.delete_image(id).await;
    }
    for id in deleted_illustrations {
        image_store.delete_illustration(id).await;
    }
}
