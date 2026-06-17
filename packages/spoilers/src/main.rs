use crate::ports::emoji::EmojiStore;
use crate::adapters::services::{card_source_init, card_storage_init, emoji_store_init, image_store_init};
use crate::ports::source::CardSource;
use crate::ports::storage::Storage;
#[cfg(feature = "local-dev")]
use dotenv::dotenv;
use crate::domain::images::save_images;

pub mod adapters;
pub mod ports;
pub mod domain;

#[tokio::main]
async fn main() {
    #[cfg(feature = "local-dev")]
    dotenv().ok();

    env_logger::init();

    let source = card_source_init();
    let storage = card_storage_init().await;
    let image_store = image_store_init();
    let emoji_store = emoji_store_init();
    emoji_store.get_emojis().await;

    // let sets = source.get_recent_sets().await;
    // let set_volumes = storage.get_set_volumes(sets).await;
    // let cards = source.fetch_cards_for_outdated_sets(&set_volumes).await;
    // if cards.is_empty() {
    //     log::info!("No available cards found");
    //     return;
    // }
    //
    // storage.upsert_cards(&cards).await;
    // save_images(&cards, &image_store, &source).await;
}
