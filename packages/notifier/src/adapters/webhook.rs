use crate::ports::spoilers::{SpoilerError, SpoilerSender};
use async_trait::async_trait;
use cards_sdk::SubscriptionId;
use contracts::card::Card;
use discord_embeds::create_embed;
use serenity::Error as SerenityError;
use serenity::builder::{Builder, CreateAttachment, ExecuteWebhook};
use serenity::http::{Http, HttpError};
use serenity::model::id::WebhookId;
use std::env;

pub struct DiscordWebhookSender {
    http: Http,
}

impl DiscordWebhookSender {
    #[must_use]
    pub fn new() -> Self {
        let token = env::var("BOT_TOKEN").expect("BOT_TOKEN wasn't in env vars");
        Self {
            http: Http::new(&token),
        }
    }
}

impl Default for DiscordWebhookSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SpoilerSender for DiscordWebhookSender {
    async fn send(
        &self,
        sub_id: SubscriptionId,
        token: &str,
        card: &Card,
        image: Vec<u8>,
    ) -> Result<(), SpoilerError> {
        let embed = create_embed(card).await;
        let attachment = CreateAttachment::bytes(image, format!("{}.png", card.image_id()));
        let builder = ExecuteWebhook::new().embed(embed).add_file(attachment);

        builder
            .execute(&self.http, (WebhookId::new(sub_id.into()), token, false))
            .await
            .map_err(|e| match &e {
                SerenityError::Http(HttpError::UnsuccessfulRequest(response)) => {
                    SpoilerError::Status(response.status_code.as_u16())
                }
                _ => SpoilerError::Request(e.to_string()),
            })?;

        Ok(())
    }
}
