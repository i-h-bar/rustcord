use crate::ports::emoji::{EmojiMetaData, SetEmoji, SymbolEmoji};
use crate::ports::image_store::{Illustration, Image};
use crate::ports::storage::{CardInfo, Set};
use async_trait::async_trait;

#[async_trait]
pub trait CardSource {
    async fn get_recent_sets(&self) -> Vec<Set>;
    async fn get_all_sets(&self) -> Vec<Set>;

    async fn fetch_cards_for_sets(&self, sets: &[Set]) -> Vec<CardInfo>;
    async fn fetch_all_cards(&self) -> Vec<CardInfo>;

    async fn get_image(&self, card: &CardInfo) -> Option<Image>;
    async fn get_illustration(&self, card: &CardInfo) -> Option<Illustration>;

    async fn download_image(&self, url: &str) -> Option<Vec<u8>>;

    async fn fetch_missing_set_symbols(&self, current: &[EmojiMetaData]) -> Vec<SetEmoji>;
    async fn fetch_missing_card_symbols(&self, current: &[EmojiMetaData]) -> Vec<SymbolEmoji>;
}
