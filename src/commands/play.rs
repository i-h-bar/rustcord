use serenity::all::{CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption};
use serenity::prelude::*;

pub(crate) async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
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
