pub mod discord;

use crate::adapters::drivers::discord::client::Discord;
use crate::domain::app::App;
use crate::ports::drivers::client::Client;
use crate::ports::services::cache::Cache;
use crate::ports::services::card_store::CardStore;
use crate::ports::services::image_store::ImageStore;

pub async fn create_client<IS, CS, C>(app: App<IS, CS, C>) -> impl Client
where
    IS: ImageStore + Send + Sync + 'static,
    CS: CardStore + Send + Sync + 'static,
    C: Cache + Send + Sync + 'static,
{
    Discord::new(app).await
}
