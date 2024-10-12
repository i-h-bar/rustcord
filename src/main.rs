#![allow(warnings)]
extern crate core;

use std::env;

use dotenv::dotenv;
use serenity::all::{GatewayIntents, Message, Ready};
use serenity::async_trait;
use serenity::client::EventHandler;
use serenity::prelude::*;
use sqlx::{Executor, Row};

use crate::mtg::MTG;

mod mtg;
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
        } else if msg.content == "!ping" {
            utils::send("Pong!", &msg, &ctx).await
        } else {
            self.mtg.find_cards(&msg, &ctx).await;
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
    let token = env::var("BOT_TOKEN").expect("Bot token wasn't in env vars");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let handler = Handler::new().await;
    let mut client = serenity::Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        log::error!("Error starting client - {why:?}")
    }
}
