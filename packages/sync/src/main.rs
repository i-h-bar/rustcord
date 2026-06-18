use crate::adapters::services::{
    card_source_init, card_storage_init, emoji_store_init, image_store_init,
};

use crate::domain::spoilers;
#[cfg(feature = "local-dev")]
use dotenv::dotenv;

pub mod adapters;
pub mod domain;
pub mod ports;

#[tokio::main]
async fn main() {
    #[cfg(feature = "local-dev")]
    dotenv().ok();

    env_logger::init();

    let source = card_source_init();
    let storage = card_storage_init().await;
    let image_store = image_store_init();
    let emoji_store = emoji_store_init();

    spoilers::sync(&source, &storage, &image_store, &emoji_store).await;
}
