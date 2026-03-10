use serenity::builder::CreateCommand;

pub fn register() -> CreateCommand {
    CreateCommand::new("give_up").description("Give up on the current game")
}
