pub mod discord;

use crate::adapters::drivers::discord::client::Discord;
use crate::domain::app::App;
use crate::ports::drivers::client::Client;
use crate::ports::services::cache::Cache;
use crate::ports::services::card_store::CardStore;
use crate::ports::services::image_store::ImageStore;
use crate::ports::services::spoiler_subscription::SpoilerSubscription;
use cards_sdk::SpoilerQueue;

pub async fn create_client<IS, CS, C, Sub>(app: App<IS, CS, C, Sub>) -> impl Client
where
    IS: ImageStore + Send + Sync + 'static,
    CS: CardStore + SpoilerQueue + Send + Sync + 'static,
    C: Cache + Send + Sync + 'static,
    Sub: SpoilerSubscription + Send + Sync + 'static,
{
    Discord::new(app).await
}
