use crate::db::Psql;
use crate::game::state::{Difficulty, GameState};
use crate::mtg::images::ImageFetcher;
use crate::utils::fuzzy_match_set_name;
use serenity::all::{
    CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, ResolvedValue,
};
use serenity::prelude::*;

pub(crate) async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<(), serenity::Error> {
    let options = interaction.data.options();
    let db = Psql::get().ok_or(serenity::Error::Other("Error contacting cards database."))?;
    let random_card = if options.is_empty() {
        db.random_distinct_card().await
    } else {
        let set_name = options
            .first()
            .ok_or(serenity::Error::Other("No first option"))?;
        let set_name = match set_name.value {
            ResolvedValue::String(name) => Ok(name),
            _ => Err(serenity::Error::Other("")),
        }?;

        let set_name = fuzzy_match_set_name(set_name)
            .await
            .ok_or(serenity::Error::Other("Unknown set name"))?;
        db.random_card_from_set(&set_name).await
    };

    if let Some(card) = random_card {
        let image_fetcher = ImageFetcher::get().ok_or(serenity::Error::Other(""))?;
        let (Some(illustration), _) = image_fetcher.fetch_illustration(&card).await else {
            return Err(serenity::Error::Other("Cannot fetch illustration data."));
        };

        let game_state = GameState::from(card, Difficulty::Easy);

        let front = game_state.to_embed();
        let response = CreateInteractionResponseMessage::new()
            .add_file(illustration)
            .add_embed(front);

        let response = CreateInteractionResponse::Message(response);
        interaction.create_response(&ctx.http, response).await?;
    } else {
        return Err(serenity::Error::Other("Could not respond to interaction."));
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("play")
        .description("Guess the card")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "set",
                "What set to choose the card from",
            )
            .required(false),
        )
}
