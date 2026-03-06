use crate::adapters::inbound::discord::utils::embed::create_embed;
use crate::adapters::inbound::discord::utils::message::{build_flip_button, build_set_dropdown, build_similar_dropdown};
use crate::domain::dto::search_result::SearchResultDto;
use crate::ports::inbound::client::{MessageInteraction, MessageInteractionError};
use async_trait::async_trait;
use serenity::all::{
    ComponentInteraction, Context, CreateActionRow, CreateAttachment, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

pub const PICK_PRINT_ID: &str = "pick-print-id";
pub const SIMILAR_ID: &str = "similar-id";
pub const FLIP: &str = "flip:";

pub struct DiscordComponentInteraction {
    ctx: Context,
    component: ComponentInteraction,
}

impl DiscordComponentInteraction {
    pub fn new(ctx: Context, component: ComponentInteraction) -> Self {
        Self { ctx, component }
    }
}

#[async_trait]
impl MessageInteraction for DiscordComponentInteraction {
    async fn send_card(&self, result: SearchResultDto) -> Result<(), MessageInteractionError> {
        let card = result.card();
        let front_image = CreateAttachment::bytes(
            result.image().bytes(),
            format!("{}.png", &card.front_image_id()),
        );
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
        self.component
            .create_response(&self.ctx, CreateInteractionResponse::UpdateMessage(message))
            .await
            .map_err(|e| MessageInteractionError::new(e.to_string()))
    }

    async fn reply(&self, message: String) -> Result<(), MessageInteractionError> {
        self.component
            .create_response(
                &self.ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(message),
                ),
            )
            .await
            .map_err(|e| MessageInteractionError::new(e.to_string()))
    }
}
