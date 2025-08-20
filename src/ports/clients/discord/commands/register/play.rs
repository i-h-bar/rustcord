use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption};

pub fn register() -> CreateCommand {
    CreateCommand::new("play")
        .description("Start a guess the card game")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "set",
                "What set to choose the card from",
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "difficulty",
                "what difficulty do you want to play at?",
            )
            .add_string_choice("Easy", "Easy")
            .add_string_choice("Medium", "Medium")
            .add_string_choice("Hard", "Hard")
            .required(false),
        )
}
