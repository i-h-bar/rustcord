use crate::adapters::inbound::create_client;
use crate::adapters::outbound::cache::init_cache;
use crate::adapters::outbound::card_store::init_card_store;
use crate::adapters::outbound::image_store::init_image_store;
use crate::domain::app::App;
use crate::ports::inbound::client::Client;

#[cfg(debug_assertions)]
use dotenv::dotenv;

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
