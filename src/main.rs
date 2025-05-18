extern crate core;

use std::env;

use dotenv::dotenv;
use serenity::all::{
    Command, CreateInteractionResponse, CreateInteractionResponseMessage, GatewayIntents,
    Interaction, Message, Ready,
};
use serenity::async_trait;
use serenity::client::EventHandler;
use serenity::prelude::*;

use crate::db::Psql;
use crate::help::HELP;
use crate::mtg::search::MTG;

mod commands;
mod db;
pub mod emoji;
mod game;
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

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            log::info!(
                "Received command: {:?} from {}",
                command.data.name,
                command.channel_id
            );

            let content = match command.data.name.as_str() {
                "play" => match commands::play::run(&ctx, &command).await {
                    Err(e) => Some(e.to_string()),
                    Ok(_) => None,
                },
                _ => Some(format!(
                    "Unknown command: {:?} from {}",
                    command.data.name, command.channel_id
                )),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    log::warn!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        match Command::create_global_command(&ctx, commands::play::register()).await {
            Err(error) => log::warn!("Could not create command {:?}", error),
            Ok(command) => log::info!("Created play command: {:?}", command),
        };

        log::info!("Bot ready!")
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    Psql::init().await;

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
