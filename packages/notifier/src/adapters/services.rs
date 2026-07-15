#[must_use]
pub async fn card_storage_init() -> cards_sdk::Postgres {
    cards_sdk::Postgres::create().await
}
