use std::env;

use dotenv::dotenv;
use serenity::all::{Command, GatewayIntents, Interaction, Message, Ready};
use serenity::async_trait;
use serenity::client::EventHandler;
use serenity::prelude::*;

use dbs::psql::Psql;
use utils::help::HELP;

mod commands;
mod dbs;
mod game;
pub mod mtg;
mod utils;
pub mod image_store;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == ctx.cache.current_user().id || msg.author.bot {
            return;
        } else if msg.content == "!help" {
            utils::send(HELP, &msg, &ctx).await;
        } else {
            for card in mtg::search::parse_message(&msg.content).await {
                mtg::card_response(card, &msg, &ctx).await;
            }
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        if let Err(err) = Command::create_global_command(&ctx, commands::play::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created play command");
        }

        if let Err(err) = Command::create_global_command(&ctx, commands::guess::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created guess command");
        }

        if let Err(err) = Command::create_global_command(&ctx, commands::help::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created help command");
        }

        if let Err(err) = Command::create_global_command(&ctx, commands::search::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created search command");
        }

        if let Err(err) = Command::create_global_command(&ctx, commands::give_up::register()).await
        {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created give_up command");
        }

        log::info!("Bot ready!");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if command.user.bot {
                return;
            }

            log::info!(
                "Received command: {:?} from {}",
                command.data.name,
                command.channel_id
            );

            match command.data.name.as_str() {
                "help" => commands::help::run(&ctx, &command).await,
                "search" => commands::search::run(&ctx, &command).await,
                "play" => commands::play::run(&ctx, &command).await,
                "guess" => commands::guess::run(&ctx, &command).await,
                "give_up" => commands::give_up::run(&ctx, &command).await,
                _ => (),
            }
        }
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

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        log::error!("Error starting client - {why:?}");
    }
}
