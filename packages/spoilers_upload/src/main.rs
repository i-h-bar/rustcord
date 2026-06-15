use crate::ports::source::CardSource;
use crate::adapters::services::card_source_init;
#[cfg(feature = "local-dev")]
use dotenv::dotenv;

pub mod adapters;
pub mod ports;

#[tokio::main]
async fn main() {
    #[cfg(feature = "local-dev")]
    dotenv().ok();

    let card_source = card_source_init();
    let sets = card_source.get_recent_sets().await;

    // TODO! Filter out sets that match number of cards already in database

    let _cards = card_source.get_cards_from_set(&sets).await;

}
