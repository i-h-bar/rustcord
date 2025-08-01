use serenity::all::CreateCommand;

pub fn register() -> CreateCommand {
    CreateCommand::new("help").description("Instructions on how to use the bot.")
}
