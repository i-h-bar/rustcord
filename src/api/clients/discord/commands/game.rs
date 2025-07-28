use crate::api::clients::discord::utils::{create_embed, create_game_embed};
use crate::api::clients::{GameInteraction, MessageInterationError};
use crate::domain::functions::game::state::{Difficulty, GameState};
use crate::spi::image_store::Images;
use async_trait::async_trait;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateInteractionResponse,
    CreateInteractionResponseMessage, MessageBuilder,
};

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
    fn id(&self) -> String {
        self.command.channel_id.to_string()
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
