use crate::adapters::services::discord::Discord;
use crate::adapters::services::file_system::FileSystem;
use crate::adapters::services::psql::Postgres;
use crate::adapters::services::scryfall::Scryfall;
use crate::ports::emoji::EmojiStore;
use crate::ports::image_store::ImageStore;
use crate::ports::source::CardSource;
use crate::ports::storage::Storage;

pub mod psql;
mod scryfall;
pub mod file_system;
pub mod discord;

#[must_use]
pub fn card_source_init() -> impl CardSource {
    Scryfall::new()
}

#[must_use]
pub async fn card_storage_init() -> impl Storage {
    Postgres::create().await
}

#[must_use]
pub fn image_store_init() -> impl ImageStore {
    FileSystem::new()
}

#[must_use]
pub fn emoji_store_init() -> impl EmojiStore {
    Discord::new()
}
