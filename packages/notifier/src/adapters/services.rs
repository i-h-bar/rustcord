use crate::adapters::images::FileSystem;
use crate::adapters::webhook::DiscordWebhookSender;
use crate::ports::images::ImageStore;
use crate::ports::spoilers::SpoilerSender;

#[must_use]
pub async fn card_storage_init() -> cards_sdk::Postgres {
    cards_sdk::Postgres::create().await
}

#[must_use]
pub fn spoiler_sender_init() -> impl SpoilerSender {
    DiscordWebhookSender::new()
}

#[must_use]
pub fn init_image_store() -> impl ImageStore {
    FileSystem::create()
}
