use crate::adapters::inbound::discord::components::interaction::{PICK_PRINT_ID, FLIP};
use crate::adapters::inbound::discord::utils::embed::create_embed;
use crate::domain::card::Card;
use crate::domain::set::Set;
use crate::ports::inbound::client::{MessageInteraction, MessageInteractionError};
use crate::ports::outbound::image_store::Images;
use async_trait::async_trait;
use serenity::all::{ButtonStyle, CommandInteraction, Context, CreateActionRow, CreateAttachment, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption};
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
        
        let mut message = CreateInteractionResponseMessage::new()
            .add_file(front_image);

        if let Some(sets) = sets {
            let options: Vec<CreateSelectMenuOption> = sets
                .iter()
                .take(25) // Discord's hard limit
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
        self.send_message(message).await?;

        Ok(())
    }

    async fn reply(&self, message: String) -> Result<(), MessageInteractionError> {
        let message = CreateInteractionResponseMessage::new().content(message);
        self.send_message(message).await?;

        Ok(())
    }
}
