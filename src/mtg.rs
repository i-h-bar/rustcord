use crate::mtg::search::CardAndImage;
use crate::{utils, Handler};
use serenity::all::{Context, Message};

mod db;
mod images;
pub mod search;

impl<'a> Handler {
    pub async fn card_response(
        &'a self,
        card: &Option<CardAndImage>,
        msg: &Message,
        ctx: &Context,
    ) {
        match card {
            None => utils::send("Failed to find card :(", &msg, &ctx).await,
            Some((card, (front_image, back_image))) => {
                self.send_image(front_image, &card.front_name, &msg, &ctx).await;
                if let Some(name) = &card.back_name {
                    self.send_image(back_image, name, &msg, &ctx).await;
                }
            }
        }
    }

    async fn send_image(&self, image: &Option<Vec<u8>>, name: &str, msg: &Message, ctx: &Context) {
        if let Some(image) = image {
            utils::send_image(
                &image,
                &format!("{}.png", &name),
                None,
                &msg,
                &ctx,
            )
                .await;
        } else {
            utils::send("Failed to find card :(", &msg, &ctx).await;
        }
    }
}
