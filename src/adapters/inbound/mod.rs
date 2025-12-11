pub mod discord;

use crate::adapters::inbound::discord::client::Discord;
use crate::domain::app::App;
use crate::ports::inbound::client::Client;
use crate::ports::outbound::cache::Cache;
use crate::ports::outbound::card_store::CardStore;
use crate::ports::outbound::image_store::ImageStore;

pub async fn create_client<IS, CS, C>(app: App<IS, CS, C>) -> impl Client
where
    IS: ImageStore + Send + Sync + 'static,
    CS: CardStore + Send + Sync + 'static,
    C: Cache + Send + Sync + 'static,
{
    Discord::new(app).await
}
