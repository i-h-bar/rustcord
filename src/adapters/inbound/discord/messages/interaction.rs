use crate::adapters::inbound::discord::utils::embed::create_embed;
use crate::adapters::inbound::discord::utils::message::{build_flip_button, build_set_dropdown};
use crate::domain::card::Card;
use crate::domain::set::Set;
use crate::ports::inbound::client::{MessageInteraction, MessageInteractionError};
use crate::ports::outbound::image_store::Images;
use async_trait::async_trait;
use serenity::all::{Context, CreateActionRow, CreateAttachment, CreateMessage, Message};
use tokio::time::Instant;

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

    async fn send_message(&self, message: CreateMessage) -> Result<(), MessageInteractionError> {
        let start = Instant::now();
        match self
            .msg
            .channel_id
            .send_message(&self.ctx.http, message)
            .await
        {
            Err(why) => Err(MessageInteractionError::new(why.to_string())),
            Ok(response) => {
                log::info!(
                    "Discord RTT took {}ms to send the message to {:?}",
                    start.elapsed().as_millis(),
                    response.channel_id.to_string()
                );
                Ok(())
            }
        }
    }
}

#[async_trait]
impl MessageInteraction for DiscordMessageInteration {
    async fn send_card(
        &self,
        card: Card,
        images: Images,
        sets: Option<Vec<Set>>,
    ) -> Result<(), MessageInteractionError> {
        let front_image =
            CreateAttachment::bytes(images.front, format!("{}.png", card.front_image_id()));

        let mut components: Vec<CreateActionRow> = Vec::with_capacity(2);

        let mut message = CreateMessage::new().add_file(front_image);
        if let Some(component) = build_set_dropdown(sets) {
            components.push(component);
        }

        if let Some(component) = build_flip_button(&card) {
            components.push(component);
        }

        if !components.is_empty() {
            message = message.components(components);
        }

        let embed = create_embed(card);
        message = message.add_embed(embed);
        self.send_message(message).await?;

        Ok(())
    }

    async fn reply(&self, message: String) -> Result<(), MessageInteractionError> {
        self.msg
            .channel_id
            .say(&self.ctx, message)
            .await
            .map_err(|_| MessageInteractionError::new(String::from("Failed to send message")))?;

        Ok(())
    }
}
