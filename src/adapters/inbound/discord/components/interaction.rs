use crate::adapters::inbound::discord::utils::embed::create_embed;
use crate::domain::card::Card;
use crate::domain::set::Set;
use crate::ports::inbound::client::{MessageInteraction, MessageInteractionError};
use crate::ports::outbound::image_store::Images;
use async_trait::async_trait;
use serenity::all::{
    ComponentInteraction, Context, CreateActionRow, CreateAttachment, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};

pub const PICK_PRINT_ID: &str = "pick-print-id";

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
    async fn send_card(
        &self,
        card: Card,
        images: Images,
        sets: Option<Vec<Set>>,
    ) -> Result<(), MessageInteractionError> {
        let front_image =
            CreateAttachment::bytes(images.front, format!("{}.png", card.front_image_id()));

        let (front, _) = create_embed(card);
        let mut message = CreateInteractionResponseMessage::new()
            .add_file(front_image)
            .add_embed(front);

        if let Some(sets) = sets {
            let options: Vec<CreateSelectMenuOption> = sets
                .iter()
                .take(25)
                .map(|s| CreateSelectMenuOption::new(s.name(), s.card_id().to_string()))
                .collect();
            let menu =
                CreateSelectMenu::new(PICK_PRINT_ID, CreateSelectMenuKind::String { options })
                    .placeholder("Select a print...");
            let row = CreateActionRow::SelectMenu(menu);
            message = message.components(vec![row]);
        }

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
