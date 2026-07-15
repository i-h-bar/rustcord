use crate::ports::services::cache::Cache;
use crate::ports::services::card_store::CardStore;
use crate::ports::services::image_store::ImageStore;
use crate::ports::services::spoiler_subscription::SpoilerSubscription;
use cards_sdk::SpoilerQueue;

pub struct App<IS, CS, C, Sub> {
    pub image_store: IS,
    pub card_store: CS,
    pub cache: C,
    pub sub: Sub,
}

impl<IS, CS, C, Sub> App<IS, CS, C, Sub>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + SpoilerQueue + Send + Sync,
    C: Cache + Send + Sync,
    Sub: SpoilerSubscription + Send + Sync,
{
    pub fn new(image_store: IS, card_store: CS, cache: C, sub: Sub) -> Self {
        Self {
            image_store,
            card_store,
            cache,
            sub,
        }
    }
}
