use crate::ports::services::spoiler_subscription::{
    Subscription, SpoilerSubError, SpoilerSubscription,
};
use async_trait::async_trait;
use secrecy::ExposeSecret;
use serenity::all::{ChannelId, CreateWebhook, Http, WebhookId};
use std::env;
use std::sync::Arc;

pub struct DiscordWebhookRegistrar {
    http: Arc<Http>,
}

#[async_trait]
impl SpoilerSubscription for DiscordWebhookRegistrar {
    fn create() -> Self {
        let token = env::var("BOT_TOKEN").expect("BOT_TOKEN wasn't in env vars");
        Self {
            http: Arc::new(Http::new(&token)),
        }
    }

    async fn create_subscription(
        &self,
        channel_id: u64,
    ) -> Result<Subscription, SpoilerSubError> {
        let channel = ChannelId::new(channel_id);
        let builder = CreateWebhook::new("Spoiler Notifications");

        let webhook = channel
            .create_webhook(&self.http, builder)
            .await
            .map_err(|e| SpoilerSubError::new(e.to_string()))?;

        let token = webhook
            .token
            .as_ref()
            .map(|t| t.expose_secret().clone())
            .ok_or_else(|| SpoilerSubError::new("created webhook has no token"))?;

        Ok(Subscription {
            id: i64::try_from(webhook.id.get())
                .map_err(|_| SpoilerSubError::new("webhook id doesn't fit in i64"))?,
            token,
        })
    }

    async fn delete_subscription(&self, webhook_id: u64) -> Result<(), SpoilerSubError> {
        self.http
            .delete_webhook(WebhookId::new(webhook_id), None)
            .await
            .map_err(|e| SpoilerSubError::new(e.to_string()))
    }
}
