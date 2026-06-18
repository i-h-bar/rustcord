use crate::domain::utils::images::save_images;
use crate::ports::emoji::EmojiStore;
use crate::ports::image_store::ImageStore;
use crate::ports::source::CardSource;
use crate::ports::storage::Storage;

async fn sync(
    source: impl CardSource,
    storage: impl Storage,
    image_store: impl ImageStore,
    emoji_store: impl EmojiStore,
) {
    let current_emojis = emoji_store.get_emojis().await;

    let sets = source.get_recent_sets().await;

    let new_set_symbols = source.fetch_missing_set_symbols(&current_emojis).await;
    emoji_store.upload_set_emojis(new_set_symbols).await;

    let set_volumes = storage.get_set_volumes(sets).await;
    let cards = source.fetch_cards_for_outdated_sets(&set_volumes).await;
    if cards.is_empty() {
        log::info!("No available cards found");
        return;
    }

    storage.upsert_cards(&cards).await;
    save_images(&cards, &image_store, &source).await;
}
