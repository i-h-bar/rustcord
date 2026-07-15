use crate::adapters::drivers::discord::utils::message::{
    build_flip_button, build_set_dropdown, build_similar_dropdown,
};
use crate::ports::drivers::client::{MessageInteraction, MessageInteractionError};
use async_trait::async_trait;
use contracts::search_result::SearchResultDto;
use discord_embeds::create_embed;
use serenity::all::{
    Context, CreateActionRow, CreateAttachment, CreateMessage, Mentionable, Message,
};
use tokio::time::Instant;

fn bot_not_mentioned_warning(user: &impl Mentionable, bot: &impl Mentionable) -> String {
    format!(
        "{} You didn't @mention me, which for now still works — but Discord is revoking our \
        message content privilege on 31st July 2026, because apparently letting a card bot \
        read card names is a security risk. From then on, inline queries without a mention \
        will get you nothing but silence. Get ahead of it: mention {} now, or switch to the \
        `/search` command, which will keep working either way.",
        user.mention(),
        bot.mention()
    )
}

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
        let message = if self.msg.mentions_me(&self.ctx.http).await.unwrap_or(false) {
            message
        } else {
            let bot_id = self.ctx.cache.current_user().id;
            message.content(bot_not_mentioned_warning(&self.msg.author.id, &bot_id))
        };

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
    async fn send_card(&self, result: SearchResultDto) -> Result<(), MessageInteractionError> {
        let card = result.card();
        let front_image =
            CreateAttachment::bytes(result.image().bytes(), format!("{}.png", card.image_id()));

        let mut components: Vec<CreateActionRow> = Vec::with_capacity(2);

        let mut message = CreateMessage::new().add_file(front_image);
        if let Some(component) = build_set_dropdown(result.printings()).await {
            components.push(component);
        }

        if let Some(component) = build_similar_dropdown(result.similar_cards()).await {
            components.push(component);
        }

        if let Some(component) = build_flip_button(card) {
            components.push(component);
        }

        if !components.is_empty() {
            message = message.components(components);
        }

        let embed = create_embed(card).await;
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
