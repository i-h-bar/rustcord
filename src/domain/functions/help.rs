use crate::domain::utils::help::HELP;
use crate::ports::clients::MessageInteraction;

pub async fn run<I: MessageInteraction>(interaction: &I) {
    if let Err(why) = interaction.reply(HELP.into()).await {
        log::error!("couldn't create interaction response: {:?}", why);
    };
}
