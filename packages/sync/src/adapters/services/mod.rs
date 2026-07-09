use crate::adapters::services::discord::Discord;
use crate::adapters::services::file_system::FileSystem;
use crate::adapters::services::scryfall::Scryfall;
use crate::ports::emoji::EmojiStore;
use crate::ports::image_store::ImageStore;
use crate::ports::source::CardSource;
use cards_sdk::WriteRepository;

pub mod discord;
pub mod file_system;
mod scryfall;

/// Connections reserved for other long-lived consumers (e.g. `bot`) when
/// `sync` claims the rest of the available Postgres connection budget for
/// its bulk/spoiler upsert fan-out.
const RESERVE_FOR_OTHER_CONSUMERS: u32 = 10;

#[must_use]
pub fn card_source_init() -> impl CardSource {
    Scryfall::new()
}

#[must_use]
pub async fn card_storage_init() -> impl WriteRepository {
    cards_sdk::Postgres::create_for_batch(RESERVE_FOR_OTHER_CONSUMERS).await
}

#[must_use]
pub fn image_store_init() -> impl ImageStore {
    FileSystem::new()
}

#[must_use]
pub fn emoji_store_init() -> impl EmojiStore {
    Discord::new()
}