extern crate core;

use std::env;

use dotenv::dotenv;
use serenity::all::{GatewayIntents, Message, Ready};
use serenity::async_trait;
use serenity::client::EventHandler;
use serenity::prelude::*;

use crate::db::PSQL;
use crate::help::HELP;
use crate::mtg::search::MTG;

mod db;
pub mod emoji;
mod help;
pub mod mtg;
mod utils;

struct Handler {
    mtg: MTG,
}

impl Handler {
    async fn new() -> Self {
        Self {
            mtg: MTG::new().await,
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == ctx.cache.current_user().id {
            return;
        } else if msg.content == "!help" {
            utils::send(HELP, &msg, &ctx).await
        } else {
            for card in self.mtg.parse_message(&msg.content).await {
                self.card_response(card, &msg, &ctx).await;
            }
        }
    }

    async fn ready(&self, _: Context, _: Ready) {
        log::info!("Bot ready!")
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    PSQL::init().await;

    let token = env::var("BOT_TOKEN").expect("Bot token wasn't in env vars");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let handler = Handler::new().await;
    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        log::error!("Error starting client - {why:?}")
    }
}
