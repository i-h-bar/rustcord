use crate::ports::emoji::EmojiStore;
use crate::ports::source::CardSource;
use unicode_normalization::UnicodeNormalization;

#[must_use]
pub fn normalise_name(name: &str) -> String {
    let normalised: String = name
        .nfkc()
        .collect::<String>()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();

    if normalised.len() < 2 {
        format!("{normalised}_")
    } else {
        normalised
    }
}

pub async fn sync(source: &impl CardSource, emoji_store: &impl EmojiStore) {
    let Some(current_emojis) = emoji_store.get_emojis().await else {
        return;
    };

    let card_symbols = source.fetch_missing_card_symbols(&current_emojis).await;
    emoji_store.upload_symbol_emojis(card_symbols).await;

    let new_set_symbols = source.fetch_missing_set_symbols(&current_emojis).await;
    emoji_store.upload_set_symbols(new_set_symbols).await;
}
