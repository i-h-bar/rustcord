use crate::utils::help::HELP;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

pub async fn run(ctx: &Context, interaction: &CommandInteraction) {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(HELP)
            .ephemeral(true),
    );
    if let Err(why) = interaction.create_response(&ctx.http, response).await {
        log::error!("couldn't create interaction response: {:?}", why);
    };
}

pub fn register() -> CreateCommand {
    CreateCommand::new("help").description("Instructions on how to use the bot.")
}
