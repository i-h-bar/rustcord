use crate::adapters::inbound::discord::components::interaction::PICK_PRINT_ID;
use crate::adapters::inbound::discord::utils::embed::create_embed;
use crate::domain::card::Card;
use crate::domain::set::Set;
use crate::ports::inbound::client::{MessageInteraction, MessageInteractionError};
use crate::ports::outbound::image_store::Images;
use async_trait::async_trait;
use serenity::all::{
    Context, CreateActionRow, CreateAttachment, CreateMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, Message,
};
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

        let front = create_embed(card);
        let mut message = CreateMessage::new().add_file(front_image).add_embed(front);
        message = if let Some(sets) = sets {
            let options: Vec<CreateSelectMenuOption> = sets
                .iter()
                .take(25) // Discord's hard limit
                .map(|s| CreateSelectMenuOption::new(s.name(), s.card_id().to_string()))
                .collect();
            let menu =
                CreateSelectMenu::new(PICK_PRINT_ID, CreateSelectMenuKind::String { options })
                    .placeholder("Select a print...");
            let row = CreateActionRow::SelectMenu(menu);
            message.components(vec![row])
        } else {
            message
        };

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
