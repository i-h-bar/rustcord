use crate::mtg::db::FuzzyFound;
use crate::mtg::search::CardAndImage;
use crate::{utils, Handler};
use serenity::all::{Context, CreateAttachment, CreateMessage, Embed, Message};
use serenity::builder::CreateEmbed;

pub mod db;
mod images;
pub mod search;

impl<'a> Handler {
    pub async fn card_response(&'a self, card: Option<CardAndImage>, msg: &Message, ctx: &Context) {
        match card {
            None => utils::send("Failed to find card :(", &msg, &ctx).await,
            Some((card, (front_image, back_image))) => {
                self.send_embed(card, front_image, back_image, &msg, &ctx)
                    .await;
            }
        }
    }

    async fn send_image(&self, image: &Option<Vec<u8>>, name: &str, msg: &Message, ctx: &Context) {
        if let Some(image) = image {
            utils::send_image(&image, &format!("{}.png", &name), None, &msg, &ctx).await;
        } else {
            utils::send("Failed to find card :(", &msg, &ctx).await;
        }
    }

    async fn send_embed(
        &self,
        card: FuzzyFound,
        front_image: Option<Vec<u8>>,
        back_image: Option<Vec<u8>>,
        msg: &Message,
        ctx: &Context,
    ) {
        let message = if let Some(front_image) = front_image {
            CreateMessage::new().add_file(CreateAttachment::bytes(
                front_image,
                format!("{}.png", card.front_image_id),
            ))
        } else {
            CreateMessage::new()
        }
        .add_embed(card.to_embed());

        match msg.channel_id.send_message(&ctx.http, message).await {
            Err(why) => {
                log::warn!("Error sending image - {why:?}")
            }
            Ok(_) => {
                log::info!("Sent embed")
            }
        }
    }
}
