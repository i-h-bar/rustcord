use crate::api::clients::MessageInteraction;
use crate::utils::help::HELP;
use serenity::all::CreateCommand;

pub async fn run<I: MessageInteraction>(interaction: &I) {
    if let Err(why) = interaction.reply(HELP.into()).await {
        log::error!("couldn't create interaction response: {:?}", why);
    };
}

pub fn register() -> CreateCommand {
    CreateCommand::new("help").description("Instructions on how to use the bot.")
}
