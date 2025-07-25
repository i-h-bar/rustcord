use crate::app::App;
use crate::cache::Cache;
use crate::card_store::CardStore;
use crate::clients::MessageInteraction;
use crate::image_store::ImageStore;
use crate::query::QueryParams;
use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption};

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub async fn search_command<I: MessageInteraction>(
        &self,
        interaction: &I,
        query_params: QueryParams,
    ) {
        let card = self.find_card(query_params).await;
        if let Some((card, images)) = card {
            if let Err(why) = interaction.send_card(card, images).await {
                log::warn!("Error sending card from search command: {}", why);
            };
        } else if let Err(why) = interaction
            .reply(String::from("Could not find card :("))
            .await
        {
            log::warn!(
                "Error the failed to find card message from search command: {}",
                why
            );
        }
    }
}

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
