use crate::ports::emoji::EmojiStore;
use crate::ports::source::CardSource;

pub async fn sync(source: &impl CardSource, emoji_store: &impl EmojiStore) {
    let Some(current_emojis) = emoji_store.get_emojis().await else {
        return;
    };

    let card_symbols = source.fetch_missing_card_symbols(&current_emojis).await;
    emoji_store.upload_symbol_emojis(card_symbols).await;

    let new_set_symbols = source.fetch_missing_set_symbols(&current_emojis).await;
    emoji_store.upload_set_symbols(new_set_symbols).await;
}
