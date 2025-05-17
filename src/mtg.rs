use crate::mtg::db::FuzzyFound;
use crate::mtg::search::CardAndImage;
use crate::{utils, Handler};
use serenity::all::{Context, CreateAttachment, CreateMessage, Message};

pub mod db;
mod images;
pub mod search;

impl Handler {
    pub async fn card_response(&self, card: Option<CardAndImage>, msg: &Message, ctx: &Context) {
        match card {
            None => utils::send("Failed to find card :(", msg, ctx).await,
            Some((card, (front_image, back_image))) => {
                self.send_embed(card, front_image, back_image, msg, ctx)
                    .await;
            }
        }
    }

    async fn send_embed(
        &self,
        card: FuzzyFound,
        front_image: Option<CreateAttachment>,
        back_image: Option<CreateAttachment>,
        msg: &Message,
        ctx: &Context,
    ) {
        let (front, back) = card.to_embed();
        let message = if let Some(front_image) = front_image {
            CreateMessage::new().add_file(front_image)
        } else {
            CreateMessage::new()
        }
        .add_embed(front);

        utils::send_message(message, msg, ctx).await;

        if let Some(back) = back {
            let message = if let Some(back_image) = back_image {
                CreateMessage::new().add_file(back_image)
            } else {
                CreateMessage::new()
            }
            .add_embed(back);

            utils::send_message(message, msg, ctx).await;
        }
    }
}
