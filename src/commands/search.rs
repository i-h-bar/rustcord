use serenity::all::{CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage};
use crate::{mtg, utils};
use crate::mtg::db::QueryParams;
use crate::utils::parse;

pub async fn run(ctx: &Context, interaction: &CommandInteraction) {
    let query_params = match parse::options::<QueryParams>(interaction.data.options()) {
        Ok(params) => params,
        Err(err) => {
            log::warn!("{}", err);
            return;
        }
    };
    
    let card = mtg::search::find_card(query_params).await;
    if let Some((card, (front_image, back_image))) = card {
        let (front, back) = card.to_embed();
        let mut message = if let Some(front_image) = front_image {
            CreateInteractionResponseMessage::new().add_file(front_image)
        } else {
            CreateInteractionResponseMessage::new()
        }
            .add_embed(front);

        if let Some(back) = back {
            message = if let Some(back_image) = back_image {
                message.add_file(back_image)
            } else {
                message
            }.add_embed(back);
        };

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


pub fn register() -> CreateCommand {
    CreateCommand::new("search")
        .description("Search for a card")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "name",
                "Name of the card",
            )
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
