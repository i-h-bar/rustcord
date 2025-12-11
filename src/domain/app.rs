use crate::ports::outbound::cache::Cache;
use crate::ports::outbound::card_store::CardStore;
use crate::ports::outbound::image_store::ImageStore;

pub struct App<IS, CS, C> {
    pub image_store: IS,
    pub card_store: CS,
    pub cache: C,
}

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub fn new(image_store: IS, card_store: CS, cache: C) -> Self {
        Self {
            image_store,
            card_store,
            cache,
        }
    }
}
