#![allow(warnings)]
extern crate core;

use std::env;
use std::time::Duration;

use dotenv::dotenv;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use serenity::all::{GatewayIntents, Message};
use serenity::async_trait;
use serenity::client::EventHandler;
use serenity::prelude::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

mod mtg;
mod utils;

struct Handler {
    http_client: Client,
    card_regex: Regex,
    pg_pool: Pool<Postgres>,
}

impl Handler {
    async fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Rust Discord Bot"));
        let http_client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::new(30, 0))
            .build()
            .expect("Failed HTTP Client build");

        let card_regex = Regex::new(r"\[\[(.*?)]]").expect("Invalid regex");

        let uri = env::var("PSQL_URI").expect("Postgres uri wasn't in env vars");
        let pg_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&uri)
            .await
            .expect("Failed Postgres connection");

        Self {
            http_client,
            card_regex,
            pg_pool,
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
            self.find_cards(&msg, &ctx).await;
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
