pub mod search;

use crate::card_store::CardStore;
use crate::image_store::ImageStore;
use crate::utils::help::HELP;
use crate::{commands, mtg, utils};
use async_trait::async_trait;
use serenity::all::{Command, Context, EventHandler, Interaction, Message, Ready};

pub struct App<IS, CS> {
    pub(crate) image_store: IS,
    pub(crate) card_store: CS,
}

impl<IS, CS> App<IS, CS>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
{
    pub(crate) fn new(image_store: IS, card_store: CS) -> Self {
        App {
            image_store,
            card_store,
        }
    }
}

#[async_trait]
impl<IS, CS> EventHandler for App<IS, CS>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
{
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == ctx.cache.current_user().id || msg.author.bot {
            return;
        } else if msg.content == "!help" {
            utils::send(HELP, &msg, &ctx).await;
        } else {
            for card in self.parse_message(&msg.content).await {
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
                "search" => self.search_command(&ctx, &command).await,
                "play" => self.play_command(&ctx, &command).await,
                "guess" => self.guess_command(&ctx, &command).await,
                "give_up" => self.give_up_command(&ctx, &command).await,
                _ => (),
            }
        }
    }
}
