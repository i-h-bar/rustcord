use crate::app::App;
use crate::cache::Cache;
use crate::card_store::CardStore;
use crate::clients::{GameInteraction, MessageInteraction, MessageInterationError};
use crate::commands::guess::GuessOptions;
use crate::commands::play::PlayOptions;
use crate::game::state::{Difficulty, GameState};
use crate::image_store::{ImageStore, Images};
use crate::mtg::card::Card;
use crate::query::QueryParams;
use crate::utils::colours::get_colour_identity;
use crate::utils::emoji::add_emoji;
use crate::utils::help::HELP;
use crate::utils::parse::{ParseError, ResolveOption};
use crate::utils::{italicise_reminder_text, parse, REGEX_COLLECTION};
use crate::{commands, mtg};
use async_trait::async_trait;
use regex::Captures;
use serenity::all::{
    Command, CommandInteraction, Context, CreateAttachment, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EventHandler,
    Interaction, Message, MessageBuilder, Ready, ResolvedValue,
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

            log::info!(
                "Received command: {:?} from {}",
                command.data.name,
                command.channel_id.to_string(),
            );

            match command.data.name.as_str() {
                "help" => {
                    let interaction = DiscordCommand { ctx, command };
                    commands::help::run(&interaction).await;
                }
                "search" => {
                    let interaction = DiscordCommand { ctx, command };
                    let query_params =
                        match parse::options::<QueryParams>(interaction.command.data.options()) {
                            Ok(params) => params,
                            Err(err) => {
                                log::warn!("{}", err);
                                return;
                            }
                        };
                    self.search_command(&interaction, query_params).await;
                }
                "play" => {
                    let interaction = DiscordCommandInteraction { ctx, command };
                    let options =
                        match parse::options::<PlayOptions>(interaction.command.data.options()) {
                            Ok(options) => options,
                            Err(err) => {
                                log::warn!("{}", err);
                                return;
                            }
                        };
                    self.play_command(&interaction, options).await;
                }
                "guess" => {
                    let interaction = DiscordCommandInteraction { ctx, command };
                    let guess_options = match parse::options(interaction.command.data.options()) {
                        Ok(value) => value,
                        Err(err) => {
                            log::warn!("Failed to parse guess: {}", err);
                            return;
                        }
                    };
                    self.guess_command(&interaction, guess_options).await;
                }
                "give_up" => {
                    let interaction = DiscordCommandInteraction { ctx, command };
                    self.give_up_command(&interaction).await;
                }
                _ => (),
            }
        }
    }
}

struct DiscordCommand {
    ctx: Context,
    command: CommandInteraction,
}

impl DiscordCommand {
    async fn send_message(
        &self,
        message: CreateInteractionResponseMessage,
    ) -> Result<(), MessageInterationError> {
        if let Err(why) = self
            .command
            .create_response(&self.ctx, CreateInteractionResponse::Message(message))
            .await
        {
            Err(MessageInterationError(why.to_string()))
        } else {
            log::info!("Sent message to {:?}", self.command.channel_id.to_string());
            Ok(())
        }
    }
}

#[async_trait]
impl MessageInteraction for DiscordCommand {
    async fn send_card(
        &self,
        card: Card,
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
        let message = CreateInteractionResponseMessage::new()
            .add_file(front_image)
            .add_embed(front);

        self.send_message(message).await?;

        if let Some(back) = back {
            if let Some(back_image) = back_image {
                let message = CreateInteractionResponseMessage::new()
                    .add_file(back_image)
                    .add_embed(back);
                self.send_message(message).await?;
            }
        }

        Ok(())
    }

    async fn reply(&self, message: String) -> Result<(), MessageInterationError> {
        let message = CreateInteractionResponseMessage::new().content(message);
        self.send_message(message).await?;

        Ok(())
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
        card: Card,
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
impl GameInteraction for DiscordCommandInteraction {
    fn id(&self) -> String {
        self.command.channel_id.to_string()
    }
    async fn send_card(
        &self,
        card: Card,
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
        if let Err(why) = self.command.create_response(&self.ctx.http, response).await {
            log::warn!("couldn't create interaction: {}", why);
        }

        if let Some(back) = back {
            if let Some(back_image) = back_image {
                let response = CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_file(back_image)
                        .add_embed(back),
                );
                if let Err(why) = self.command.create_response(&self.ctx.http, response).await {
                    log::warn!("couldn't create interaction: {}", why);
                }
            }
        }

        Ok(())
    }

    async fn game_failed_message(
        &self,
        state: GameState,
        images: Images,
    ) -> Result<(), MessageInterationError> {
        let image = CreateAttachment::bytes(
            images.front,
            format!("{}.png", state.card().front_image_id()),
        );
        let number_of_guesses = state.number_of_guesses();
        let guess_plural = if number_of_guesses > 1 {
            "guesses"
        } else {
            "guess"
        };

        let message = MessageBuilder::new()
            .push(format!(
                "You have all failed after {number_of_guesses} {guess_plural}!",
            ))
            .build();

        let (embed, _) = create_embed(state.card);

        let response = CreateInteractionResponseMessage::new()
            .add_file(image)
            .add_embed(embed)
            .content(message);

        let response = CreateInteractionResponse::Message(response);
        if self
            .command
            .create_response(&self.ctx.http, response)
            .await
            .is_err()
        {
            return Err(MessageInterationError(String::from(
                "couldn't create interaction",
            )));
        }

        Ok(())
    }

    async fn send_win_message(
        &self,
        state: GameState,
        images: Images,
    ) -> Result<(), MessageInterationError> {
        let image = CreateAttachment::bytes(
            images.front,
            format!("{}.png", state.card().front_image_id()),
        );

        let number_of_guesses = state.number_of_guesses();
        let guess_plural = if number_of_guesses > 1 {
            "guesses"
        } else {
            "guess"
        };

        let message = MessageBuilder::new()
            .mention(&self.command.user)
            .push(format!(
                " has won after {number_of_guesses} {guess_plural}!",
            ))
            .build();

        let (embed, _) = create_embed(state.card);

        let response = CreateInteractionResponseMessage::new()
            .add_file(image)
            .add_embed(embed)
            .content(message);

        let response = CreateInteractionResponse::Message(response);
        if self
            .command
            .create_response(&self.ctx.http, response)
            .await
            .is_err()
        {
            return Err(MessageInterationError(String::from(
                "Failed to send message",
            )));
        }

        Ok(())
    }

    async fn send_guess_wrong_message(
        &self,
        state: GameState,
        images: Images,
        guess: String,
    ) -> Result<(), MessageInterationError> {
        let illustration = if let Some(illustration_id) = state.card().front_illustration_id() {
            CreateAttachment::bytes(images.front, format!("{illustration_id}.png",))
        } else {
            log::warn!("Card had no illustration id");
            return Err(MessageInterationError(String::from(
                "Card had no illustration id",
            )));
        };

        let embed = create_game_embed(&state.card, state.multiplier(), state.guesses());

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
        if let Err(why) = self.command.create_response(&self.ctx.http, response).await {
            log::warn!("couldn't create interaction: {}", why);
        }

        Ok(())
    }
    async fn send_new_game_message(
        &self,
        state: GameState,
        images: Images,
    ) -> Result<(), MessageInterationError> {
        let Some(illustration_id) = state.card().front_illustration_id() else {
            return Err(MessageInterationError(String::from(
                "Failed to get image id",
            )));
        };

        let illustration = CreateAttachment::bytes(images.front, format!("{illustration_id}.png",));
        let difficulty = state.difficulty();
        let set_name = state.card().set_name();
        let message = match difficulty {
            Difficulty::Hard => format!("Difficulty is set to `{difficulty}`."),
            _ => format!("Difficulty is set to `{difficulty}`. This card is from `{set_name}`"),
        };

        let embed = create_game_embed(&state.card, state.multiplier(), state.guesses());
        let response = CreateInteractionResponseMessage::new()
            .content(message)
            .add_file(illustration)
            .add_embed(embed);

        let response = CreateInteractionResponse::Message(response);
        if let Err(why) = self.command.create_response(&self.ctx.http, response).await {
            log::error!("couldn't create interaction response: {:?}", why);
        }

        Ok(())
    }

    async fn reply(&self, message: String) -> Result<(), MessageInterationError> {
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(message)
                .ephemeral(true),
        );
        if self
            .command
            .create_response(&self.ctx.http, response)
            .await
            .is_err()
        {
            return Err(MessageInterationError(String::from(
                "couldn't create interaction",
            )));
        }

        Ok(())
    }
}

fn create_game_embed(card: &Card, multiplier: usize, guesses: usize) -> CreateEmbed {
    let mut embed = CreateEmbed::default()
        .attachment(format!(
            "{}.png",
            card.front_illustration_id.unwrap_or_default()
        ))
        .title("????")
        .description("????")
        .footer(CreateEmbedFooter::new(format!("🖌️ - {}", card.artist)));

    if guesses > multiplier {
        let mana_cost = REGEX_COLLECTION
            .symbols
            .replace_all(&card.front_mana_cost, |cap: &Captures| add_emoji(cap));
        let title = format!("????        {mana_cost}");
        embed = embed
            .title(title)
            .colour(get_colour_identity(&card.front_colour_identity));
    }

    if guesses > multiplier * 2 {
        embed = embed.description(card.rules_text());
    }

    embed
}

fn create_embed(card: Card) -> (CreateEmbed, Option<CreateEmbed>) {
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

impl ResolveOption for PlayOptions {
    fn resolve(option: Vec<(&str, ResolvedValue)>) -> Result<Self, ParseError> {
        let mut set: Option<String> = None;
        let mut difficulty: Difficulty = Difficulty::Medium;

        for (name, value) in option {
            match name {
                "set" => {
                    set = match value {
                        ResolvedValue::String(card) => Some(card.to_string()),
                        _ => return Err(ParseError::new("set ResolvedValue was not a string")),
                    };
                }
                "difficulty" => {
                    difficulty = match value {
                        ResolvedValue::String(difficulty_string) => match difficulty_string {
                            "Easy" => Difficulty::Easy,
                            "Medium" => Difficulty::Medium,
                            "Hard" => Difficulty::Hard,
                            default => {
                                return Err(ParseError::new(&format!(
                                    "Could not parse {default} into difficulty"
                                )))
                            }
                        },
                        _ => {
                            return Err(ParseError::new(
                                "difficulty ResolvedValue was not a string",
                            ))
                        }
                    };
                }
                _ => {}
            }
        }

        Ok(PlayOptions::new(set, difficulty))
    }
}

impl ResolveOption for QueryParams {
    fn resolve(options: Vec<(&str, ResolvedValue)>) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        let mut card_name = None;
        let mut set_name = None;
        let mut set_code = None;
        let mut artist = None;

        for (name, value) in options {
            match name {
                "name" => {
                    card_name = match value {
                        ResolvedValue::String(card) => Some(card.to_string()),
                        _ => return Err(ParseError::new("Name was not a string")),
                    }
                }
                "set" => {
                    let set = match value {
                        ResolvedValue::String(set) => set.to_string(),
                        _ => return Err(ParseError::new("Name was not a string")),
                    };
                    if set.chars().count() < 5 {
                        set_code = Some(set);
                    } else {
                        set_name = Some(set);
                    }
                }
                "artist" => {
                    artist = match value {
                        ResolvedValue::String(artist) => Some(artist.to_string()),
                        _ => return Err(ParseError::new("Artist was not a string")),
                    }
                }
                _ => {}
            }
        }

        let Some(name) = card_name else {
            return Err(ParseError::new("No name found in query params"));
        };

        Ok(Self::new(artist, name, set_code, set_name))
    }
}

impl ResolveOption for GuessOptions {
    fn resolve(options: Vec<(&str, ResolvedValue)>) -> Result<Self, ParseError> {
        let Some((_, guess)) = options.first() else {
            return Err(ParseError::new("Could not get first option"));
        };

        let guess = match guess {
            ResolvedValue::String(guess) => (*guess).to_string(),
            _ => return Err(ParseError::new("ResolvedValue was not a string")),
        };

        Ok(GuessOptions::new(guess))
    }
}
