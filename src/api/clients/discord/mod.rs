mod commands;
mod messages;
mod utils;

use crate::api::clients::discord::commands::game::DiscordCommandInteraction;
use crate::api::clients::discord::commands::interaction::DiscordCommand;
use crate::api::clients::discord::commands::register::{give_up, guess, help, play, search};
use crate::api::clients::discord::messages::interaction::DiscordMessageInteration;
use crate::api::clients::MessageInteraction;
use crate::domain::app::App;
use crate::domain::game::play::PlayOptions;
use crate::domain::query::QueryParams;
use crate::domain::{card, game};
use crate::spi::cache::Cache;
use crate::spi::card_store::CardStore;
use crate::spi::image_store::ImageStore;
use crate::utils::help::HELP;
use crate::utils::parse;
use async_trait::async_trait;
use serenity::all::{Command, Context, EventHandler, Interaction, Message, Ready};

#[async_trait]
impl<IS, CS, C> EventHandler for App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == ctx.cache.current_user().id || msg.author.bot {
            return;
        } else if msg.content == "!help" {
            let interaction = DiscordMessageInteration::new(ctx, msg);
            if let Err(why) = interaction.reply(HELP.to_string()).await {
                log::error!("Error sending help message: {:?}", why);
            };
        } else {
            let interaction = DiscordMessageInteration::new(ctx, msg);
            for card in self.parse_message(interaction.content()).await {
                card::card_response(card, &interaction).await;
            }
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        if let Err(err) = Command::create_global_command(&ctx, play::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created play command");
        }

        if let Err(err) = Command::create_global_command(&ctx, guess::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created guess command");
        }

        if let Err(err) = Command::create_global_command(&ctx, help::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created help command");
        }

        if let Err(err) = Command::create_global_command(&ctx, search::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created search command");
        }

        if let Err(err) = Command::create_global_command(&ctx, give_up::register()).await {
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
                command.channel_id.to_string(),
            );

            match command.data.name.as_str() {
                "help" => {
                    let interaction = DiscordCommand::new(ctx, command);
                    game::help::run(&interaction).await;
                }
                "search" => {
                    let query_params = match parse::options::<QueryParams>(command.data.options()) {
                        Ok(params) => params,
                        Err(err) => {
                            log::warn!("{}", err);
                            return;
                        }
                    };
                    let interaction = DiscordCommand::new(ctx, command);
                    self.search(&interaction, query_params).await;
                }
                "play" => {
                    let options = match parse::options::<PlayOptions>(command.data.options()) {
                        Ok(options) => options,
                        Err(err) => {
                            log::warn!("{}", err);
                            return;
                        }
                    };
                    let interaction = DiscordCommandInteraction::new(ctx, command);
                    self.play_command(&interaction, options).await;
                }
                "guess" => {
                    let guess_options = match parse::options(command.data.options()) {
                        Ok(value) => value,
                        Err(err) => {
                            log::warn!("Failed to parse guess: {}", err);
                            return;
                        }
                    };
                    let interaction = DiscordCommandInteraction::new(ctx, command);
                    self.guess_command(&interaction, guess_options).await;
                }
                "give_up" => {
                    let interaction = DiscordCommandInteraction::new(ctx, command);
                    self.give_up_command(&interaction).await;
                }
                _ => (),
            }
        }
    }
}
