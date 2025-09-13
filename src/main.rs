use crate::ports::clients::Client;

#[cfg(debug_assertions)]
use dotenv::dotenv;

use crate::domain::app::App;
use crate::ports::clients::create_client;
use adapters::cache::init_cache;
use adapters::card_store::init_card_store;
use adapters::image_store::init_image_store;

mod adapters;
mod domain;
mod ports;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    {
        dotenv().ok();
    }

    env_logger::init();
    let card_store = init_card_store().await;
    let image_store = init_image_store();
    let cache = init_cache();

    let app = App::new(image_store, card_store, cache);
    let mut client = create_client(app).await;

    client.run().await;
}
