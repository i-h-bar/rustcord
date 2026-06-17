use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct EmojiImage(String);

#[derive(Serialize, Deserialize)]
pub struct EmojiMetaData {
    id: String,
    name: String,
}

pub struct Emoji {
    id: String,
    name: String,
    image: EmojiImage,
}

#[async_trait]
pub trait EmojiStore {
    async fn get_emojis(&self) -> Vec<EmojiMetaData>;
    async fn upload_emojis(&self, emojis: Vec<Emoji>);
}