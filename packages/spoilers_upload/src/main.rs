use crate::ports::storage::Storage;
use crate::ports::source::CardSource;
use crate::adapters::services::{card_storage_init, card_source_init};
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

    let _cards = source.fetch_cards_for_outdated_sets(&set_volumes).await;
    let _ = 9;
}
