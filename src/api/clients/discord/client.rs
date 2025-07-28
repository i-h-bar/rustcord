use crate::api::clients::Client;
use crate::domain::app::App;
use crate::spi::cache::Cache;
use crate::spi::card_store::CardStore;
use crate::spi::image_store::ImageStore;
use async_trait::async_trait;
use serenity::all::GatewayIntents;
use serenity::Client as DiscordClient;
use std::env;

pub struct Discord(DiscordClient);

impl Discord {
    pub async fn new<IS, CS, C>(app: App<IS, CS, C>) -> Self
    where
        IS: ImageStore + Send + Sync + 'static,
        CS: CardStore + Send + Sync + 'static,
        C: Cache + Send + Sync + 'static,
    {
        let token = env::var("BOT_TOKEN").expect("Bot token wasn't in env vars");
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let client = DiscordClient::builder(&token, intents)
            .event_handler(app)
            .await
            .expect("Error creating client");

        Self(client)
    }
}

#[async_trait]
impl Client for Discord {
    async fn run(&mut self) {
        if let Err(why) = self.0.start().await {
            log::error!("Error starting client - {why:?}");
        }
    }
}
