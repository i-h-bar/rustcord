#![allow(warnings)]
extern crate core;

use std::collections::HashSet;
use std::env;
use std::time::Duration;

use crate::mtg::MTG;
use dotenv::dotenv;
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use serenity::all::{GatewayIntents, Message};
use serenity::async_trait;
use serenity::client::EventHandler;
use serenity::prelude::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, Pool, Postgres, Row};

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
}

#[tokio::main]
async fn main() {
    dotenv().ok();
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
        println!("Error starting client - {why:?}")
    }
}
