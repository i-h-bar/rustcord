use crate::adapters::services::scryfall::Scryfall;
use crate::ports::source::CardSource;

mod scryfall;

pub fn card_source_init() -> impl CardSource {
    Scryfall::new()
}
