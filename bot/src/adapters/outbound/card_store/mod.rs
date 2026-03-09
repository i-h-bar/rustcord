pub mod postgres;

use crate::adapters::outbound::card_store::postgres::Postgres;
use crate::ports::outbound::card_store::CardStore;

pub async fn init_card_store() -> impl CardStore {
    Postgres::create().await
}
