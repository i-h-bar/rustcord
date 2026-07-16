use crate::ports::services::spoiler_subscription::{
    SpoilerSubError, SpoilerSubscription, Subscription,
};
use async_trait::async_trait;
use cards_sdk::{ChannelId, SubscriptionId};
use secrecy::ExposeSecret;
use serenity::all::{
    ChannelId as DiscordChannelId, CreateAttachment, CreateWebhook, Http,
    WebhookId as DiscordWebhookId,
};
use serenity::http::HttpError;
use serenity::Error as SerenityError;
use std::env;

pub struct DiscordWebhookRegistrar {
    http: Http,
}

#[async_trait]
impl SpoilerSubscription for DiscordWebhookRegistrar {
    fn create() -> Self {
        let token = env::var("BOT_TOKEN").expect("BOT_TOKEN wasn't in env vars");
        Self {
            http: Http::new(&token),
        }
    }

    async fn create_subscription(
        &self,
        channel_id: ChannelId,
    ) -> Result<Subscription, SpoilerSubError> {
        let channel = DiscordChannelId::new(channel_id.into());
        let mut builder = CreateWebhook::new("Card Bot Spoiler Notifications");

        // Best-effort: give the webhook the bot's own avatar so it doesn't
        // show up with Discord's blank default one. Not fatal if this fails
        // — the webhook is still useful without a matching avatar.
        if let Ok(bot_user) = self.http.get_current_user().await {
            if let Some(avatar_url) = bot_user.avatar_url() {
                if let Ok(avatar) = CreateAttachment::url(&self.http, &avatar_url).await {
                    builder = builder.avatar(&avatar);
                }
            }
        }

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
            id: SubscriptionId::from(webhook.id.get()),
            token,
        })
    }

    async fn delete_subscription(&self, sub_id: SubscriptionId) -> Result<(), SpoilerSubError> {
        self.http
            .delete_webhook(DiscordWebhookId::new(sub_id.into()), None)
            .await
            .map_err(|e| match &e {
                SerenityError::Http(HttpError::UnsuccessfulRequest(response)) => {
                    SpoilerSubError::with_status(e.to_string(), response.status_code.as_u16())
                }
                _ => SpoilerSubError::new(e.to_string()),
            })
    }
}
