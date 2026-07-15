use crate::ports::webhook::{WebhookError, WebhookSender};
use async_trait::async_trait;
use serde::Serialize;
use serenity::builder::CreateEmbed;

#[derive(Serialize)]
struct WebhookPayload {
    embeds: Vec<CreateEmbed>,
}

pub struct ReqwestWebhookSender {
    client: reqwest::Client,
    base_url: String,
}

impl Default for ReqwestWebhookSender {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestWebhookSender {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://discord.com/api/webhooks".to_string(),
        }
    }
}

#[async_trait]
impl WebhookSender for ReqwestWebhookSender {
    async fn send(
        &self,
        webhook_id: i64,
        webhook_token: &str,
        embed: CreateEmbed,
    ) -> Result<(), WebhookError> {
        let url = format!("{}/{}/{}", self.base_url, webhook_id, webhook_token);
        let resp = self
            .client
            .post(&url)
            .json(&WebhookPayload {
                embeds: vec![embed],
            })
            .send()
            .await
            .map_err(|e| WebhookError::Request(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(WebhookError::Status(resp.status().as_u16()))
        }
    }
}
