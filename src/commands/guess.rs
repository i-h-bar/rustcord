use crate::game::state::GameState;
use crate::mtg::images::ImageFetcher;
use crate::redis::Redis;
use crate::utils::fuzzy;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateAllowedMentions, CreateCommand,
    CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateMessage, MessageBuilder, ResolvedValue,
};
use std::fmt::format;

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    let game_state_string: String = Redis::instance()
        .ok_or(serenity::Error::Other("Error contacting cards database."))?
        .get(interaction.channel_id.to_string())
        .await
        .ok_or(serenity::Error::Other("No game found"))?;
    let game_state: GameState =
        ron::from_str(&game_state_string).map_err(|_| serenity::Error::Other(""))?;

    let guess = match interaction
        .data
        .options()
        .first()
        .ok_or(serenity::Error::Other("No card given in the guess"))?
        .value
    {
        ResolvedValue::String(card) => Ok(card),
        _ => Err(serenity::Error::Other("")),
    }?;

    if fuzzy::jaro_winkler(&guess, game_state.card()) > 0.75 {
        let (Some(image), _) = ImageFetcher::get()
            .ok_or(serenity::Error::Other("No card image"))?
            .fetch(game_state.card())
            .await
        else {
            return Err(serenity::Error::Other("No card image"));
        };

        let embed = game_state.to_full_embed();

        let message = MessageBuilder::new()
            .mention(&interaction.user)
            .push(" Won!")
            .build();
        if let Err(why) = interaction.channel_id.say(&ctx.http, &message).await {
            log::warn!("Error sending message: {why:?}");
        }

        let response = CreateInteractionResponseMessage::new()
            .add_file(image)
            .add_embed(embed);

        let response = CreateInteractionResponse::Message(response);
        interaction.create_response(&ctx.http, response).await?;

        if let Err(why) = Redis::instance()
            .ok_or(serenity::Error::Other("Error contacting redis"))?
            .delete(interaction.channel_id.to_string())
            .await
        {
            log::warn!(
                "Error deleting key: '{}' from redis the response: {:?}",
                interaction.channel_id.to_string(),
                why
            );
        };
    } else {
        let response = CreateInteractionResponseMessage::new()
            .content(format!("'{}' was not the correct card", guess));

        let response = CreateInteractionResponse::Message(response);
        interaction.create_response(&ctx.http, response).await?;
    }

    Ok(())
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
