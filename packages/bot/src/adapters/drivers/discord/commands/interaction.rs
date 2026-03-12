use crate::adapters::drivers::discord::utils::embed::create_embed;
use crate::adapters::drivers::discord::utils::message::{
    build_flip_button, build_set_dropdown, build_similar_dropdown,
};
use crate::ports::drivers::client::{MessageInteraction, MessageInteractionError};
use async_trait::async_trait;
use contracts::search_result::SearchResultDto;
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
    async fn send_card(&self, result: SearchResultDto) -> Result<(), MessageInteractionError> {
        let card = result.card();
        let front_image =
            CreateAttachment::bytes(result.image().bytes(), format!("{}.png", card.image_id()));
        let mut components: Vec<CreateActionRow> = Vec::with_capacity(2);

        let mut message = CreateInteractionResponseMessage::new().add_file(front_image);

        if let Some(component) = build_set_dropdown(result.printings()) {
            components.push(component);
        }

        if let Some(component) = build_similar_dropdown(result.similar_cards()) {
            components.push(component);
        }

        if let Some(component) = build_flip_button(card) {
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
