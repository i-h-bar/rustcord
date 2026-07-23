use crate::impl_app;

pub struct App<IS, CS, C, Sub> {
    pub image_store: IS,
    pub card_store: CS,
    pub cache: C,
    pub sub: Sub,
}

impl_app! {
    pub fn new(image_store: IS, card_store: CS, cache: C, sub: Sub) -> Self {
        Self {
            image_store,
            card_store,
            cache,
            sub,
        }
    }
}
