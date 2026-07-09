use crate::ports::services::card_store::CardStore;

pub async fn init_card_store() -> impl CardStore {
    cards_sdk::Postgres::create().await
}