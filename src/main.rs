use std::env;

use dotenv::dotenv;
use serenity::all::GatewayIntents;
use serenity::prelude::*;

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

    let token = env::var("BOT_TOKEN").expect("Bot token wasn't in env vars");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(app)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        log::error!("Error starting client - {why:?}");
    }
}
