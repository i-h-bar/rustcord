use std::env;

use dotenv::dotenv;
use regex::Regex;
use serenity::{async_trait, Client};
use serenity::all::{GatewayIntents, Message};
use serenity::client::EventHandler;
use serenity::prelude::*;

struct Handler {
    card_regex: Regex
}

impl Handler {
    fn new() -> Self {
        let card_regex = Regex::new(r"\[[a-zA-Z]+]").expect("Invalid regex");

        Handler {
            card_regex
        }
    }

    async fn send(&self, content: &str, msg: &Message, ctx: &Context) {
        if let Err(why) = msg.channel_id.say(&ctx.http, content).await {
            println!("Error sending message - {why:?}")
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == ctx.cache.current_user().id {
            return;
        } else if msg.content == "!ping" {
            self.send("Pong!", &msg, &ctx).await
        } else {
            if let Some(captures) = self.card_regex.captures(&msg.content) {
                for capture in captures.iter() {
                    let Some(card) = capture else { continue };
                    self.send(card.as_str(), &msg, &ctx).await
                }
            }
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

    let handler = Handler::new();

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Error starting client - {why:?}")
    }
}
