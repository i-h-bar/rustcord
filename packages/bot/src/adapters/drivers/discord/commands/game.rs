use crate::domain::functions::game::state::{Difficulty, GameState};
use crate::ports::drivers::client::{GameInteraction, MessageInteractionError};
use async_trait::async_trait;
use contracts::card::Card;
use contracts::image::Image;
use discord_embeds::{add_emoji, create_embed, get_colour_identity, italicise_reminder_text};
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, MessageBuilder,
};
use uuid::Uuid;

/// The guessing game's progressive-reveal embed — distinct from `/search`'s
/// `discord_embeds::create_embed`, since it hides the card name/mana
/// cost/rules text until enough wrong guesses have been made. Not part of
/// the `discord_embeds` extraction: this is game-specific presentation
/// logic, not something `notifier` or any other consumer needs.
pub async fn create_game_embed(card: &Card, multiplier: usize, guesses: usize) -> CreateEmbed {
    let mut embed = CreateEmbed::default()
        .attachment(format!(
            "{}.png",
            card.illustration_id().unwrap_or(&Uuid::default())
        ))
        .title("????")
        .description("????")
        .footer(CreateEmbedFooter::new(format!("🖌️ - {}", card.artist())));

    if guesses > multiplier {
        let mana_cost = add_emoji(card.mana_cost()).await;
        let title = format!("????        {mana_cost}");
        embed = embed
            .title(title)
            .colour(get_colour_identity(card.colour_identity()));
    }

    if guesses > multiplier * 2 {
        let stats = if let Some(power) = card.power() {
            let toughness = card.toughness().unwrap_or("0");
            format!("\n\n{power}/{toughness}")
        } else if let Some(loyalty) = card.loyalty() {
            format!("\n\n{loyalty}")
        } else if let Some(defence) = card.defence() {
            format!("\n\n{defence}")
        } else {
            String::new()
        };

        let rules_text = add_emoji(card.oracle_text()).await;
        let oracle_text = italicise_reminder_text(&rules_text);

        embed = embed.description(format!("{}\n\n{}{}", card.type_line(), oracle_text, stats));
    }

    embed
}

pub struct DiscordCommandInteraction {
    ctx: Context,
    command: CommandInteraction,
}

impl DiscordCommandInteraction {
    pub fn new(ctx: Context, command: CommandInteraction) -> Self {
        Self { ctx, command }
    }
}

#[async_trait]
impl GameInteraction for DiscordCommandInteraction {
    async fn send_guess_wrong_message(
        &self,
        state: GameState,
        images: Image,
        guess: String,
    ) -> Result<(), MessageInteractionError> {
        let illustration = if let Some(illustration_id) = state.card().illustration_id() {
            CreateAttachment::bytes(images.bytes(), format!("{illustration_id}.png"))
        } else {
            log::warn!("Card had no illustration id");
            return Err(MessageInteractionError::new(String::from(
                "Card had no illustration id",
            )));
        };

        let embed = create_game_embed(&state.card, state.multiplier(), state.guesses()).await;

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
            log::warn!("couldn't create interaction: {why}");
        }

        Ok(())
    }

    async fn send_new_game_message(
        &self,
        state: GameState,
        images: Image,
    ) -> Result<(), MessageInteractionError> {
        let Some(illustration_id) = state.card().illustration_id() else {
            return Err(MessageInteractionError::new(String::from(
                "Failed to get image id",
            )));
        };

        let illustration =
            CreateAttachment::bytes(images.bytes(), format!("{illustration_id}.png"));
        let difficulty = state.difficulty();
        let set_name = state.card().set_name();
        let message = match difficulty {
            Difficulty::Hard => format!("Difficulty is set to `{difficulty}`."),
            _ => format!("Difficulty is set to `{difficulty}`. This card is from `{set_name}`"),
        };

        let embed = create_game_embed(&state.card, state.multiplier(), state.guesses()).await;
        let response = CreateInteractionResponseMessage::new()
            .content(message)
            .add_file(illustration)
            .add_embed(embed);

        let response = CreateInteractionResponse::Message(response);
        if let Err(why) = self.command.create_response(&self.ctx.http, response).await {
            log::error!("couldn't create interaction response: {why:?}");
        }

        Ok(())
    }

    async fn send_win_message(
        &self,
        state: GameState,
        images: Image,
    ) -> Result<(), MessageInteractionError> {
        let image =
            CreateAttachment::bytes(images.bytes(), format!("{}.png", state.card().image_id()));

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

        let embed = create_embed(&state.card).await;

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
            return Err(MessageInteractionError::new(String::from(
                "Failed to send message",
            )));
        }

        Ok(())
    }

    async fn game_failed_message(
        &self,
        state: GameState,
        images: Image,
    ) -> Result<(), MessageInteractionError> {
        let image =
            CreateAttachment::bytes(images.bytes(), format!("{}.png", state.card().image_id()));
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

        let embed = create_embed(&state.card).await;

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
            return Err(MessageInteractionError::new(String::from(
                "couldn't create interaction",
            )));
        }

        Ok(())
    }
    fn id(&self) -> String {
        self.command.channel_id.to_string()
    }

    async fn reply(&self, message: String) -> Result<(), MessageInteractionError> {
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
            return Err(MessageInteractionError::new(String::from(
                "couldn't create interaction",
            )));
        }

        Ok(())
    }
}
