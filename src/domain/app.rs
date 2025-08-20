use crate::adapters::cache::Cache;
use crate::adapters::card_store::CardStore;
use crate::adapters::image_store::ImageStore;

pub struct App<IS, CS, C> {
    pub(crate) image_store: IS,
    pub(crate) card_store: CS,
    pub(crate) cache: C,
}

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub(crate) fn new(image_store: IS, card_store: CS, cache: C) -> Self {
        Self {
            image_store,
            card_store,
            cache,
        }
    }
}
