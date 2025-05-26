use crate::game::state;
use crate::mtg::images::IMAGE_FETCHER;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, MessageBuilder,
};

pub async fn run(ctx: &Context, interaction: &CommandInteraction) {
    let Some(game_state) = state::fetch(ctx, interaction).await else {
        return;
    };

    state::delete(interaction).await;

    let (Some(image), _) = IMAGE_FETCHER.fetch(game_state.card()).await else {
        log::warn!("couldn't fetch image");
        return;
    };

    let number_of_guesses = game_state.number_of_guesses();
    let guess_plural = if number_of_guesses > 1 {
        "guesses"
    } else {
        "guess"
    };

    let message = MessageBuilder::new()
        .mention(&interaction.user)
        .push(format!(
            " has given up after {} {}!",
            number_of_guesses, guess_plural
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
    };
}

pub fn register() -> CreateCommand {
    CreateCommand::new("give_up").description("Give up on the current game")
}
