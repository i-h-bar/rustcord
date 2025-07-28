use crate::api::clients::Client;

use dotenv::dotenv;

use crate::api::clients::create_client;
use crate::domain::app::App;
use spi::cache::init_cache;
use spi::card_store::init_card_store;
use spi::image_store::init_image_store;

mod api;
mod domain;
mod spi;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let card_store = init_card_store().await;
    let image_store = init_image_store().await;
    let cache = init_cache().await;

    let app = App::new(image_store, card_store, cache);
    let mut client = create_client(app).await;

    client.run().await;
}
