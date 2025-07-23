use crate::app::search::CardAndImage;
use crate::image_store::Images;
use crate::mtg::card::FuzzyFound;
use crate::utils;
use serenity::all::{Context, CreateAttachment, CreateMessage, Message};

pub mod card;

pub async fn card_response(card: Option<CardAndImage>, msg: &Message, ctx: &Context) {
    match card {
        None => utils::send("Failed to find card :(", msg, ctx).await,
        Some((card, images)) => {
            send_embed(card, images, msg, ctx).await;
        }
    }
}

async fn send_embed(card: FuzzyFound, images: Images, msg: &Message, ctx: &Context) {
    let front_image =
        CreateAttachment::bytes(images.front, format!("{}.png", card.front_image_id()));
    let back_image = if let Some(back_image) = images.back {
        card.back_image_id().map(|back_image_id| {
            CreateAttachment::bytes(back_image, format!("{back_image_id}.png"))
        })
    } else {
        None
    };

    let (front, back) = card.to_embed();
    let message = CreateMessage::new().add_file(front_image).add_embed(front);

    utils::send_message(message, msg, ctx).await;

    if let Some(back) = back {
        if let Some(back_image) = back_image {
            let message = CreateMessage::new().add_file(back_image).add_embed(back);
            utils::send_message(message, msg, ctx).await;
        }
    }
}
