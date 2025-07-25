use crate::api::clients::MessageInteraction;
use crate::utils::help::HELP;

pub async fn run<I: MessageInteraction>(interaction: &I) {
    if let Err(why) = interaction.reply(HELP.into()).await {
        log::error!("couldn't create interaction response: {:?}", why);
    };
}
