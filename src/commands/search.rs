use crate::app::App;
use crate::card_store::CardStore;
use crate::image_store::ImageStore;
use crate::query::QueryParams;
use crate::utils::parse;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateAttachment, CreateCommand,
    CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
};

impl<IS, CS> App<IS, CS>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
{
    pub async fn search_command(&self, ctx: &Context, interaction: &CommandInteraction) {
        let query_params = match parse::options::<QueryParams>(interaction.data.options()) {
            Ok(params) => params,
            Err(err) => {
                log::warn!("{}", err);
                return;
            }
        };

        let card = self.find_card(query_params).await;
        if let Some((card, images)) = card {
            let front_image =
                CreateAttachment::bytes(images.front, format!("{}.png", card.front_image_id()));
            let back_image = if let Some(back_image_id) = card.back_image_id() {
                images.back.map(|back_image| {
                    CreateAttachment::bytes(back_image, format!("{back_image_id}.png"))
                })
            } else {
                None
            };

            let (front, back) = card.to_embed();

            let mut message = CreateInteractionResponseMessage::new()
                .add_file(front_image)
                .add_embed(front);

            if let Some(back) = back {
                message = if let Some(back_image) = back_image {
                    message.add_file(back_image)
                } else {
                    message
                }
                .add_embed(back);
            }

            if let Err(why) = interaction
                .create_response(ctx, CreateInteractionResponse::Message(message))
                .await
            {
                log::error!("couldn't create interaction response: {:?}", why);
            }
        } else {
            let response = CreateInteractionResponseMessage::new()
                .content("Could not find card :(")
                .ephemeral(true);
            if let Err(why) = interaction
                .create_response(ctx, CreateInteractionResponse::Message(response))
                .await
            {
                log::error!("couldn't create interaction response: {:?}", why);
            }
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
