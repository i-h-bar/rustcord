use async_trait::async_trait;
use serde::{Deserialize, Serialize};


#[derive(Debug)]
pub struct EmojiImage(pub String);

#[derive(Serialize, Deserialize)]
pub struct EmojiMetaData {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Emoji {
    pub name: String,
    pub image: EmojiImage,
}

#[async_trait]
pub trait EmojiStore {
    async fn get_emojis(&self) -> Vec<EmojiMetaData>;
    async fn upload_emojis(&self, emojis: Vec<Emoji>);
}
