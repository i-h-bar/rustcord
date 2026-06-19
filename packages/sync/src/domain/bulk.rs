use crate::ports::emoji::EmojiStore;
use crate::ports::image_store::ImageStore;
use crate::ports::source::CardSource;
use crate::ports::storage::Storage;

pub async fn sync(
    source: &impl CardSource,
    storage: &impl Storage,
    image_store: &impl ImageStore,
    emoji_store: &impl EmojiStore,
) {
    todo!()
}