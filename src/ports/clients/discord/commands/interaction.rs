use crate::adapters::image_store::Images;
use crate::domain::card::Card;
use crate::ports::clients::discord::utils::embed::create_embed;
use crate::ports::clients::{MessageInteraction, MessageInterationError};
use async_trait::async_trait;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use tokio::time::Instant;

pub struct DiscordCommand {
    ctx: Context,
    command: CommandInteraction,
}

impl DiscordCommand {
    pub fn new(ctx: Context, command: CommandInteraction) -> Self {
        Self { ctx, command }
    }
    async fn send_message(
        &self,
        message: CreateInteractionResponseMessage,
    ) -> Result<(), MessageInterationError> {
        let start = Instant::now();
        if let Err(why) = self
            .command
            .create_response(&self.ctx, CreateInteractionResponse::Message(message))
            .await
        {
            Err(MessageInterationError(why.to_string()))
        } else {
            log::info!("Discord API took {}ms to send the message to {:?}", start.elapsed().as_millis(), self.command.channel_id.to_string());
            Ok(())
        }
    }
}

#[async_trait]
impl MessageInteraction for DiscordCommand {
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
        let message = CreateInteractionResponseMessage::new()
            .add_file(front_image)
            .add_embed(front);

        self.send_message(message).await?;

        if let Some(back) = back {
            if let Some(back_image) = back_image {
                let message = CreateInteractionResponseMessage::new()
                    .add_file(back_image)
                    .add_embed(back);
                self.send_message(message).await?;
            }
        }

        Ok(())
    }

    async fn reply(&self, message: String) -> Result<(), MessageInterationError> {
        let message = CreateInteractionResponseMessage::new().content(message);
        self.send_message(message).await?;

        Ok(())
    }
}
