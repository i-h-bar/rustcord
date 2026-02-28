use crate::adapters::inbound::discord::utils::embed::create_embed;
use crate::adapters::inbound::discord::utils::message::{build_flip_button, build_set_dropdown};
use crate::domain::card::Card;
use crate::domain::set::Set;
use crate::ports::inbound::client::{MessageInteraction, MessageInteractionError};
use crate::ports::outbound::image_store::Images;
use async_trait::async_trait;
use serenity::all::{
    CommandInteraction, Context, CreateActionRow, CreateAttachment, CreateInteractionResponse,
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
    ) -> Result<(), MessageInteractionError> {
        let start = Instant::now();
        if let Err(why) = self
            .command
            .create_response(&self.ctx, CreateInteractionResponse::Message(message))
            .await
        {
            Err(MessageInteractionError::new(why.to_string()))
        } else {
            log::info!(
                "Discord RTT took {}ms to send the message to {:?}",
                start.elapsed().as_millis(),
                self.command.channel_id.to_string()
            );
            Ok(())
        }
    }
}

#[async_trait]
impl MessageInteraction for DiscordCommand {
    async fn send_card(
        &self,
        card: Card,
        images: Images,
        sets: Option<Vec<Set>>,
    ) -> Result<(), MessageInteractionError> {
        let front_image =
            CreateAttachment::bytes(images.front, format!("{}.png", card.front_image_id()));
        let mut components: Vec<CreateActionRow> = Vec::with_capacity(2);

        let mut message = CreateInteractionResponseMessage::new().add_file(front_image);

        if let Some(component) = build_set_dropdown(sets) {
            components.push(component);
        }

        if let Some(component) = build_flip_button(&card) {
            components.push(component);
        }

        if !components.is_empty() {
            message = message.components(components);
        }

        let front = create_embed(card);
        message = message.add_embed(front);
        self.send_message(message).await?;

        Ok(())
    }

    async fn reply(&self, message: String) -> Result<(), MessageInteractionError> {
        let message = CreateInteractionResponseMessage::new().content(message);
        self.send_message(message).await?;

        Ok(())
    }
}
