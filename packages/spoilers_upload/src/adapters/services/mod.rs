use crate::adapters::services::psql::Postgres;
use crate::adapters::services::scryfall::Scryfall;
use crate::ports::source::CardSource;
use crate::ports::storage::Storage;

mod scryfall;
pub mod psql;

pub fn card_source_init() -> impl CardSource {
    Scryfall::new()
}

pub async fn card_storage_init() -> impl Storage {
    Postgres::create().await
}
