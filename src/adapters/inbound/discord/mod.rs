pub mod client;
mod commands;
mod components;
mod messages;
mod utils;

use crate::adapters::inbound::discord::commands::game::DiscordCommandInteraction;
use crate::adapters::inbound::discord::commands::interaction::DiscordCommand;
use crate::adapters::inbound::discord::commands::register::{give_up, guess, help, play, search};
use crate::adapters::inbound::discord::components::interaction::{
    DiscordComponentInteraction, FLIP, PICK_PRINT_ID,
};
use crate::adapters::inbound::discord::messages::interaction::DiscordMessageInteration;
use crate::adapters::inbound::discord::utils::help::HELP;
use crate::domain::app::App;
use crate::domain::functions::game::play::PlayOptions;
use crate::domain::query::QueryParams;
use crate::domain::{card, functions};
use crate::ports::outbound::cache::Cache;
use crate::ports::outbound::card_store::CardStore;
use crate::ports::outbound::image_store::ImageStore;
use async_trait::async_trait;
use serenity::all::{
    Command, ComponentInteractionDataKind, Context, EventHandler, Interaction, Message, Ready,
};
use utils::parse;
use uuid::Uuid;

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
            functions::help::run(&interaction, HELP).await;
        } else {
            let interaction = DiscordMessageInteration::new(ctx, msg);
            for card in self.parse_message(interaction.content()).await {
                card::card_response(card, &interaction).await;
            }
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        if let Err(err) = Command::create_global_command(&ctx, play::register()).await {
            log::warn!("Could not create command {err:?}");
        } else {
            log::info!("Created play command");
        }

        if let Err(err) = Command::create_global_command(&ctx, guess::register()).await {
            log::warn!("Could not create command {err:?}");
        } else {
            log::info!("Created guess command");
        }

        if let Err(err) = Command::create_global_command(&ctx, help::register()).await {
            log::warn!("Could not create command {err:?}");
        } else {
            log::info!("Created help command");
        }

        if let Err(err) = Command::create_global_command(&ctx, search::register()).await {
            log::warn!("Could not create command {err:?}");
        } else {
            log::info!("Created search command");
        }

        if let Err(err) = Command::create_global_command(&ctx, give_up::register()).await {
            log::warn!("Could not create command {err:?}");
        } else {
            log::info!("Created give_up command");
        }

        log::info!("Bot ready!");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                if command.user.bot {
                    return;
                }

                log::info!(
                    "Received command: {:?} from {}",
                    command.data.name,
                    command.channel_id,
                );

                match command.data.name.as_str() {
                    "help" => {
                        let interaction = DiscordCommand::new(ctx, command);
                        functions::help::run(&interaction, HELP).await;
                    }
                    "search" => {
                        let query_params =
                            match parse::options::<QueryParams>(command.data.options()) {
                                Ok(params) => params,
                                Err(err) => {
                                    log::warn!("{err}");
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
                                log::warn!("{err}");
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
                                log::warn!("Failed to parse guess: {err}");
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
            Interaction::Component(component) => {
                if component.data.custom_id == PICK_PRINT_ID {
                    if let ComponentInteractionDataKind::StringSelect { values } =
                        &component.data.kind
                    {
                        if let Some(card_id_str) = values.first() {
                            log::info!(
                                "Received Pick print command for {} from {}",
                                card_id_str,
                                component.channel_id,
                            );
                            match Uuid::parse_str(card_id_str) {
                                Ok(card_id) => {
                                    let interaction =
                                        DiscordComponentInteraction::new(ctx, component);
                                    self.select_print(&interaction, card_id).await;
                                }
                                Err(why) => log::warn!("Invalid card_id in print_select: {why}"),
                            }
                        }
                    }
                } else if component.data.custom_id.starts_with(FLIP) {
                    let id = component.data.custom_id.strip_prefix(FLIP).unwrap();
                    match Uuid::parse_str(id) {
                        Ok(id) => {
                            let interaction = DiscordComponentInteraction::new(ctx, component);
                            self.select_print(&interaction, id).await;
                        }
                        Err(why) => log::warn!("Invalid id in card flip: {why}"),
                    }
                }
            }
            _ => {}
        }
    }
}
