pub mod postgres;

use crate::adapters::services::card_store::postgres::Postgres;
use crate::ports::services::card_store::CardStore;

pub async fn init_card_store() -> impl CardStore {
    Postgres::create().await
}
