use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption};

pub fn register() -> CreateCommand {
    CreateCommand::new("search")
        .description("Search for a card")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "name", "Name of the card")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "set",
                "Constrain search to a set",
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "artist",
                "Constrain search to an artist",
            )
            .required(false),
        )
}
