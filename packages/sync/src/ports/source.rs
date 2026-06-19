use crate::ports::emoji::{Emoji, EmojiMetaData};
use crate::ports::image_store::{Illustration, Image};
use crate::ports::storage::{CardInfo, Set};
use async_trait::async_trait;
use std::collections::HashSet;
use uuid::Uuid;

#[async_trait]
pub trait CardSource {
    async fn get_recent_sets(&self) -> Vec<Set>;

    async fn fetch_cards_for_outdated_sets(&self, sets: &[(Set, HashSet<Uuid>)]) -> Vec<CardInfo>;

    async fn get_image(&self, card: &CardInfo) -> Option<Image>;
    async fn get_illustration(&self, card: &CardInfo) -> Option<Illustration>;

    async fn fetch_missing_set_symbols(&self, current: &[EmojiMetaData]) -> Vec<Emoji>;
}
