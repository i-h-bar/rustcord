use std::env;

use dotenv::dotenv;
use serenity::all::GatewayIntents;
use serenity::prelude::*;

use crate::app::App;
use crate::card_store::init_card_store;
use crate::image_store::init_image_store;

pub mod app;
mod card_store;
mod commands;
mod dbs;
mod game;
pub mod image_store;
pub mod mtg;
pub mod query;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let card_store = init_card_store().await;
    let image_store = init_image_store().await;

    let app = App::new(image_store, card_store);

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
