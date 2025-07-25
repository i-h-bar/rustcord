use crate::app::App;
use crate::cache::Cache;
use crate::card_store::CardStore;
use crate::game::state;
use crate::image_store::ImageStore;
use crate::utils::mutex;
use crate::utils::parse::{ParseError, ResolveOption};
use crate::utils::{fuzzy, normalise, parse};
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateAttachment, CreateCommand,
    CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
    MessageBuilder, ResolvedValue,
};
use crate::clients::MessageInteraction;
use crate::query::QueryParams;

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub async fn guess_command<I: MessageInteraction>(&self, interaction: &I, options: GuessOptions) {
        let channel_id = interaction.id();
        let lock = mutex::LOCKS.get(&channel_id).await;
        let _guard = lock.lock().await;
        self.run_guess(interaction, options).await;
    }

    async fn run_guess<I: MessageInteraction>(&self, interaction: &I, options: GuessOptions) {
        let GuessOptions { guess } = options;

        let Some(mut game_state) = state::fetch(interaction, &self.cache).await else {
            return;
        };
        game_state.add_guess();

        if fuzzy::jaro_winkler(&normalise(&guess), &game_state.card().front_normalised_name) > 0.75
        {
            let Ok(images) = self.image_store.fetch(game_state.card()).await else {
                log::warn!("couldn't fetch image");
                return;
            };
            let image = CreateAttachment::bytes(
                images.front,
                format!("{}.png", game_state.card().front_image_id()),
            );

            let number_of_guesses = game_state.number_of_guesses();
            let guess_plural = if number_of_guesses > 1 {
                "guesses"
            } else {
                "guess"
            };

            let message = MessageBuilder::new()
                .mention(&interaction.user)
                .push(format!(
                    " has won after {number_of_guesses} {guess_plural}!",
                ))
                .build();

            let embed = game_state.convert_to_embed();

            let response = CreateInteractionResponseMessage::new()
                .add_file(image)
                .add_embed(embed)
                .content(message);

            let response = CreateInteractionResponse::Message(response);
            if let Err(why) = interaction.create_response(&ctx.http, response).await {
                log::warn!("couldn't create interaction: {}", why);
            }

            state::delete(interaction, &self.cache).await;
        } else if game_state.number_of_guesses() >= game_state.max_guesses() {
            state::delete(interaction, &self.cache).await;

            let Ok(images) = self.image_store.fetch(game_state.card()).await else {
                log::warn!("couldn't fetch image");
                return;
            };
            let image = CreateAttachment::bytes(
                images.front,
                format!("{}.png", game_state.card().front_image_id()),
            );
            let number_of_guesses = game_state.number_of_guesses();
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

            let embed = game_state.convert_to_embed();

            let response = CreateInteractionResponseMessage::new()
                .add_file(image)
                .add_embed(embed)
                .content(message);

            let response = CreateInteractionResponse::Message(response);
            if let Err(why) = interaction.create_response(&ctx.http, response).await {
                log::warn!("couldn't create interaction: {}", why);
            }
        } else {
            let Ok(images) = self.image_store.fetch_illustration(game_state.card()).await else {
                log::warn!("couldn't fetch illustration");
                return;
            };
            let illustration =
                if let Some(illustration_id) = game_state.card().front_illustration_id() {
                    CreateAttachment::bytes(images.front, format!("{illustration_id}.png",))
                } else {
                    log::warn!("couldn't fetch illustration");
                    return;
                };

            let remaining_guesses = game_state.max_guesses() - game_state.number_of_guesses();
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
                .embed(game_state.to_embed());

            let response = CreateInteractionResponse::Message(response);
            if let Err(why) = interaction.create_response(&ctx.http, response).await {
                log::warn!("couldn't create interaction: {}", why);
            }
            state::add(&game_state, interaction, &self.cache).await;
        }
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("guess")
        .description("Guess the card")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "card",
                "The name of the card you want to guess",
            )
            .required(true),
        )
}

pub struct GuessOptions {
    guess: String,
}

impl GuessOptions {
    pub fn new(guess: String) -> Self {
        Self { guess }
    }
}
