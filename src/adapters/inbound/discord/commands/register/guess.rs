use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption};

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
