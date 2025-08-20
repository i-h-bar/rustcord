use crate::adapters::image_store::Images;
use crate::domain::card::Card;
use crate::ports::clients::discord::utils::embed::create_embed;
use crate::ports::clients::{MessageInteraction, MessageInterationError};
use async_trait::async_trait;
use serenity::all::{Context, CreateAttachment, CreateMessage, Message};

pub struct DiscordMessageInteration {
    ctx: Context,
    msg: Message,
}

impl DiscordMessageInteration {
    pub fn new(ctx: Context, msg: Message) -> Self {
        Self { ctx, msg }
    }

    pub fn content(&self) -> &str {
        &self.msg.content
    }

    async fn send_message(&self, message: CreateMessage) -> Result<(), MessageInterationError> {
        match self
            .msg
            .channel_id
            .send_message(&self.ctx.http, message)
            .await
        {
            Err(why) => Err(MessageInterationError(why.to_string())),
            Ok(response) => {
                log::info!("Sent message to {:?}", response.channel_id);
                Ok(())
            }
        }
    }
}

#[async_trait]
impl MessageInteraction for DiscordMessageInteration {
    async fn send_card(&self, card: Card, images: Images) -> Result<(), MessageInterationError> {
        let front_image =
            CreateAttachment::bytes(images.front, format!("{}.png", card.front_image_id()));
        let back_image = if let Some(back_image) = images.back {
            card.back_image_id().map(|back_image_id| {
                CreateAttachment::bytes(back_image, format!("{back_image_id}.png"))
            })
        } else {
            None
        };

        let (front, back) = create_embed(card);
        let message = CreateMessage::new().add_file(front_image).add_embed(front);

        self.send_message(message).await?;

        if let Some(back) = back {
            if let Some(back_image) = back_image {
                let message = CreateMessage::new().add_file(back_image).add_embed(back);
                self.send_message(message).await?;
            }
        }

        Ok(())
    }

    async fn reply(&self, message: String) -> Result<(), MessageInterationError> {
        self.msg
            .channel_id
            .say(&self.ctx, message)
            .await
            .map_err(|_| MessageInterationError(String::from("Failed to send message")))?;

        Ok(())
    }
}
