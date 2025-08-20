use crate::ports::clients::MessageInteraction;

pub async fn run<I: MessageInteraction>(interaction: &I, text: &str) {
    if let Err(why) = interaction.reply(text.into()).await {
        log::error!("couldn't create interaction response: {:?}", why);
    };
}
