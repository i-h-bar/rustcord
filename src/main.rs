use std::env;

use dotenv::dotenv;
use serenity::all::GatewayIntents;
use serenity::prelude::*;

use crate::app::App;
use crate::image_store::init_image_store;
use dbs::psql::Psql;

pub mod app;
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
    Psql::init().await;
    let image_store = init_image_store();

    let app = App::new(image_store);

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
