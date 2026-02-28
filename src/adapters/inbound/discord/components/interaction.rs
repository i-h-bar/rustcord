use crate::adapters::inbound::discord::utils::embed::create_embed;
use crate::domain::card::Card;
use crate::domain::set::Set;
use crate::ports::inbound::client::{MessageInteraction, MessageInteractionError};
use crate::ports::outbound::image_store::Images;
use async_trait::async_trait;
use serenity::all::{ButtonStyle, ComponentInteraction, Context, CreateActionRow, CreateAttachment, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption};

pub const PICK_PRINT_ID: &str = "pick-print-id";
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
    async fn send_card(
        &self,
        card: Card,
        images: Images,
        sets: Option<Vec<Set>>,
    ) -> Result<(), MessageInteractionError> {
        let front_image =
            CreateAttachment::bytes(images.front, format!("{}.png", card.front_image_id()));
        let mut components: Vec<CreateActionRow> = Vec::with_capacity(2);
        
        let mut message = CreateInteractionResponseMessage::new()
            .add_file(front_image);

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
            components.push(row);
        }

        if let Some(back_id) = card.back_id {
            let button = CreateButton::new(format!("{FLIP}{back_id}"))
                .label("🔁")
                .style(ButtonStyle::Secondary);
            let row = CreateActionRow::Buttons(vec![button]);
            components.push(row);
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
