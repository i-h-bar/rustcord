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
    let current_emojis = emoji_store.get_emojis().await;

    source.get_all_sets().await;

    let card_symbols = source.fetch_missing_card_symbols(&current_emojis).await;
    emoji_store.upload_symbol_emojis(card_symbols).await;

    let new_set_symbols = source.fetch_missing_set_symbols(&current_emojis).await;
    emoji_store.upload_set_symbols(new_set_symbols).await;

    let cards = source.fetch_all_cards().await;
    if cards.is_empty() {
        log::info!("No cards fetched");
        return;
    }

    let upsert_result = storage.upsert_cards(&cards).await;

    save_images(
        &upsert_result.changed_images,
        &upsert_result.changed_illustrations,
        &cards,
        &image_store,
        &source,
    )
    .await;

    for &id in &upsert_result.orphaned_images {
        image_store.delete_image(id).await;
    }
    for &id in &upsert_result.orphaned_illustrations {
        image_store.delete_illustration(id).await;
    }
    storage
        .delete_orphaned_images(&upsert_result.orphaned_images)
        .await;
    storage
        .delete_orphaned_illustrations(&upsert_result.orphaned_illustrations)
        .await;
}
