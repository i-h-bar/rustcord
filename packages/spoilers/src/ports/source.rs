use crate::ports::emoji::{Emoji, EmojiMetaData};
use crate::ports::image_store::{Image, Illustration};
use crate::ports::storage::{CardInfo, Set};
use async_trait::async_trait;

#[async_trait]
pub trait CardSource {
    async fn get_recent_sets(&self) -> Vec<Set>;

    /// Fetches cards for sets where the stored card count doesn't match the expected volume.
    /// Each entry is a `(Set, u32)` pair where the `u32` is the known card count for that set.
    /// Only sets with a volume mismatch are queried — up-to-date sets are skipped.
    async fn fetch_cards_for_outdated_sets(&self, sets: &[(Set, u32)]) -> Vec<CardInfo>;

    async fn get_image(&self, card: &CardInfo) -> Option<(Image, Option<Illustration>)>;

    async fn fetch_missing_set_symbols(&self, current: &[EmojiMetaData]) -> Vec<Emoji>;
}
