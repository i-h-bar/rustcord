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
    let card_symbols = source.fetch_missing_card_symbols(&current_emojis).await;
    emoji_store.upload_symbol_emojis(card_symbols).await;
    
    let sets = source.get_recent_sets().await;

    let new_set_symbols = source.fetch_missing_set_symbols(&current_emojis).await;
    emoji_store.upload_set_symbols(new_set_symbols).await;

    let set_ids = storage.get_existing_card_ids(sets).await;
    let cards = source.fetch_cards_for_outdated_sets(&set_ids).await;
    if cards.is_empty() {
        log::info!("No available cards found");
        return;
    }

    storage.upsert_cards(&cards).await;
    drop(storage);
    
    save_images(&cards, &image_store, &source).await;
}
