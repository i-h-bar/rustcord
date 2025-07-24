use crate::app::App;
use crate::cache::Cache;
use crate::card_store::CardStore;
use crate::clients::{MessageInteraction, MessageInterationError};
use crate::game::state;
use crate::game::state::GameState;
use crate::image_store::{ImageStore, Images};
use crate::mtg::card::FuzzyFound;
use crate::utils::colours::get_colour_identity;
use crate::utils::emoji::add_emoji;
use crate::utils::help::HELP;
use crate::utils::{italicise_reminder_text, REGEX_COLLECTION};
use crate::{commands, mtg, utils};
use async_trait::async_trait;
use regex::Captures;
use serenity::all::{
    Command, CommandInteraction, Context, CreateAttachment, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EventHandler,
    Interaction, Message, Ready,
};

#[async_trait]
impl<IS, CS, C> EventHandler for App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    async fn message(&self, ctx: Context, msg: Message) {
        let interaction = DiscordMessageInteration { ctx, msg };

        if interaction.msg.author.id == interaction.ctx.cache.current_user().id
            || interaction.msg.author.bot
        {
            return;
        } else if interaction.msg.content == "!help" {
            if let Err(why) = interaction.reply(HELP.to_string()).await {
                log::error!("Error sending help message: {:?}", why);
            };
        } else {
            for card in self.parse_message(&interaction.msg.content).await {
                mtg::card_response(card, &interaction).await;
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
            
            let interaction = DiscordCommandInteraction {
                ctx, command
            };

            log::info!(
                "Received command: {:?} from {}",
                interaction.command.data.name,
                interaction.command.channel_id
            );

            match interaction.command.data.name.as_str() {
                "help" => commands::help::run(&interaction).await,
                "search" => self.search_command(&interaction).await,
                "play" => self.play_command(&interaction).await,
                "guess" => self.guess_command(&interaction).await,
                "give_up" => self.give_up_command(&interaction).await,
                _ => (),
            }
        }
    }
}

struct DiscordMessageInteration {
    ctx: Context,
    msg: Message,
}

impl DiscordMessageInteration {
    async fn send_message(&self, message: CreateMessage) -> Result<(), MessageInterationError> {
        match self
            .msg
            .channel_id
            .send_message(&self.ctx.http, message)
            .await
        {
            Err(why) => Err(MessageInterationError(why.to_string())),
            Ok(response) => {
                log::info!("Sent message to {:?}", response.channel_id);
                Ok(())
            }
        }
    }
}

#[async_trait]
impl MessageInteraction for DiscordMessageInteration {
    async fn send_card(
        &self,
        card: FuzzyFound,
        images: Images,
    ) -> Result<(), MessageInterationError> {
        let front_image =
            CreateAttachment::bytes(images.front, format!("{}.png", card.front_image_id()));
        let back_image = if let Some(back_image) = images.back {
            card.back_image_id().map(|back_image_id| {
                CreateAttachment::bytes(back_image, format!("{back_image_id}.png"))
            })
        } else {
            None
        };

        let (front, back) = create_embed(card);
        let message = CreateMessage::new().add_file(front_image).add_embed(front);

        self.send_message(message).await?;

        if let Some(back) = back {
            if let Some(back_image) = back_image {
                let message = CreateMessage::new().add_file(back_image).add_embed(back);
                self.send_message(message).await?;
            }
        }

        Ok(())
    }

    async fn send_game_state(
        &self,
        _: &GameState,
        _: Images,
        _: &str,
    ) -> Result<(), MessageInterationError> {
        Ok(())
    }

    async fn reply(&self, message: String) -> Result<(), MessageInterationError> {
        self.msg
            .channel_id
            .say(&self.ctx, message)
            .await
            .map_err(|_| MessageInterationError(String::from("Failed to send message")))?;

        Ok(())
    }
}

struct DiscordCommandInteraction {
    ctx: Context,
    command: CommandInteraction,
}

#[async_trait]
impl MessageInteraction for DiscordCommandInteraction {
    async fn send_card(
        &self,
        card: FuzzyFound,
        images: Images,
    ) -> Result<(), MessageInterationError> {
        let front_image =
            CreateAttachment::bytes(images.front, format!("{}.png", card.front_image_id()));
        let back_image = if let Some(back_image) = images.back {
            card.back_image_id().map(|back_image_id| {
                CreateAttachment::bytes(back_image, format!("{back_image_id}.png"))
            })
        } else {
            None
        };

        let (front, back) = create_embed(card);
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .add_file(front_image)
                .add_embed(front),
        );
        if let Err(why) = self
            .command
            .create_response(&self.ctx.http, response)
            .await
        {
            log::warn!("couldn't create interaction: {}", why);
        }

        if let Some(back) = back {
            if let Some(back_image) = back_image {
                let response = CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_file(back_image)
                        .add_embed(back),
                );
                if let Err(why) = self
                    .command
                    .create_response(&self.ctx.http, response)
                    .await
                {
                    log::warn!("couldn't create interaction: {}", why);
                }
            }
        }

        Ok(())
    }

    async fn send_game_state(
        &self,
        state: &GameState,
        images: Images,
        guess: &str,
    ) -> Result<(), MessageInterationError> {
        let mut embed = CreateEmbed::default()
            .attachment(format!(
                "{}.png",
                state.card().front_illustration_id.unwrap_or_default()
            ))
            .title("????")
            .description("????")
            .footer(CreateEmbedFooter::new(format!(
                "🖌️ - {}",
                state.card().artist
            )));

        if state.guesses() > state.multiplier() {
            let mana_cost = REGEX_COLLECTION
                .symbols
                .replace_all(&state.card().front_mana_cost, |cap: &Captures| {
                    add_emoji(cap)
                });
            let title = format!("????        {mana_cost}");
            embed = embed
                .title(title)
                .colour(get_colour_identity(&state.card().front_colour_identity));
        }

        if state.guesses() > state.multiplier() * 2 {
            embed = embed.description(state.card().rules_text());
        }

        let illustration = if let Some(illustration_id) = state.card().front_illustration_id() {
            CreateAttachment::bytes(images.front, format!("{illustration_id}.png",))
        } else {
            log::warn!("Card had no illustration id");
            return Err(MessageInterationError(String::from(
                "Card had no illustration id",
            )));
        };

        let remaining_guesses = state.max_guesses() - state.number_of_guesses();
        let guess_plural = if remaining_guesses > 1 {
            "guesses"
        } else {
            "guess"
        };

        let response = CreateInteractionResponseMessage::new()
            .content(format!(
                "'{guess}' was not the correct card. You have {remaining_guesses} {guess_plural} remaining",
            ))
            .add_file(illustration)
            .embed(embed);

        let response = CreateInteractionResponse::Message(response);
        if let Err(why) = self
            .command
            .create_response(&self.ctx.http, response)
            .await
        {
            log::warn!("couldn't create interaction: {}", why);
        }

        Ok(())
    }

    async fn reply(&self, message: String) -> Result<(), MessageInterationError> {
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(message)
                .ephemeral(true),
        );
        if let Err(_) = self
            .command
            .create_response(&self.ctx.http, response)
            .await
        {
            return Err(MessageInterationError(String::from("couldn't create interaction")));
        }

        Ok(())
    }
}

fn create_embed(card: FuzzyFound) -> (CreateEmbed, Option<CreateEmbed>) {
    let stats = if let Some(power) = card.front_power {
        let toughness = card.front_toughness.unwrap_or_else(|| "0".to_string());
        format!("\n\n{power}/{toughness}")
    } else if let Some(loyalty) = card.front_loyalty {
        format!("\n\n{loyalty}")
    } else if let Some(defence) = card.front_defence {
        format!("\n\n{defence}")
    } else {
        String::new()
    };

    let front_oracle_text = REGEX_COLLECTION
        .symbols
        .replace_all(&card.front_oracle_text, |cap: &Captures| add_emoji(cap));
    let front_oracle_text = italicise_reminder_text(&front_oracle_text);

    let rules_text = format!("{}\n\n{}{}", card.front_type_line, front_oracle_text, stats);
    let mana_cost = REGEX_COLLECTION
        .symbols
        .replace_all(&card.front_mana_cost, |cap: &Captures| add_emoji(cap));
    let title = format!("{}        {}", card.front_name, mana_cost);

    let front = CreateEmbed::default()
        .attachment(format!("{}.png", card.front_image_id))
        .url(card.front_scryfall_url)
        .title(title)
        .description(rules_text)
        .colour(get_colour_identity(&card.front_colour_identity))
        .footer(CreateEmbedFooter::new(format!("🖌️ - {}", card.artist)));

    let back = if let Some(name) = card.back_name {
        let stats = if let Some(power) = card.back_power {
            let toughness = card.back_toughness.unwrap_or_else(|| "0".to_string());
            format!("\n\n{power}/{toughness}")
        } else if let Some(loyalty) = card.back_loyalty {
            format!("\n\n{loyalty}")
        } else if let Some(defence) = card.back_defence {
            format!("\n\n{defence}")
        } else {
            String::new()
        };
        let back_oracle_text = card.back_oracle_text.unwrap_or_default();
        let back_oracle_text = REGEX_COLLECTION
            .symbols
            .replace_all(&back_oracle_text, |cap: &Captures| add_emoji(cap));
        let back_oracle_text = italicise_reminder_text(&back_oracle_text);

        let back_rules_text = format!(
            "{}\n\n{}{}",
            card.back_type_line.unwrap_or_default(),
            back_oracle_text,
            stats
        );
        let title = if let Some(mana_cost) = card.back_mana_cost {
            let mana_cost = REGEX_COLLECTION
                .symbols
                .replace_all(&mana_cost, |cap: &Captures| add_emoji(cap));
            format!("{name}        {mana_cost}")
        } else {
            name
        };

        let url = card.back_scryfall_url.unwrap_or_default();
        Some(
            CreateEmbed::default()
                .attachment(format!("{}.png", card.back_image_id.unwrap_or_default()))
                .url(url)
                .title(title)
                .description(back_rules_text)
                .colour(get_colour_identity(
                    &card.back_colour_identity.unwrap_or_default(),
                ))
                .footer(CreateEmbedFooter::new(format!("🖌️ - {}", card.artist))),
        )
    } else {
        None
    };

    (front, back)
}
