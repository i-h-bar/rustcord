use crate::ports::services::card_store::CardStore;
use cards_sdk::SpoilerQueue;

pub async fn init_card_store() -> impl CardStore + SpoilerQueue {
    cards_sdk::Postgres::create().await
}
