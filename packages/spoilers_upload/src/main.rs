use crate::adapters::services::{card_source_init, card_storage_init};
use crate::ports::source::CardSource;
use crate::ports::storage::Storage;
#[cfg(feature = "local-dev")]
use dotenv::dotenv;

pub mod adapters;
pub mod ports;

#[tokio::main]
async fn main() {
    #[cfg(feature = "local-dev")]
    dotenv().ok();

    let source = card_source_init();
    let storage = card_storage_init().await;

    let sets = source.get_recent_sets().await;
    let set_volumes = storage.get_set_volumes(sets).await;
    let cards = source.fetch_cards_for_outdated_sets(&set_volumes).await;

    storage.upsert_cards(cards).await;
}
