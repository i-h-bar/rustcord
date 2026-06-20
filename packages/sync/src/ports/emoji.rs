use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::domain::utils::emoji::normalise_name;

pub struct EmojiImage(pub String);

#[derive(Serialize, Deserialize)]
pub struct EmojiMetaData {
    pub id: String,
    pub name: String,
}

pub struct SetEmoji {
    pub name: String,
    pub image: EmojiImage,
}

pub struct SymbolEmoji {
    pub name: String,
    pub image: EmojiImage,
} 

impl SymbolEmoji {
    #[must_use]
    pub fn new(name: &str, image: EmojiImage) -> Self {
        Self { name: normalise_name(name), image }
    }
}

impl EmojiName for SetEmoji {
    fn clone_name(&self) -> String {
        self.name.clone()
    }
}

impl EmojiName for SymbolEmoji {
    fn clone_name(&self) -> String {
        self.name.clone()
    }
}

pub trait EmojiName {
    fn clone_name(&self) -> String;
}

#[async_trait]
pub trait EmojiStore {
    async fn get_emojis(&self) -> Vec<EmojiMetaData>;
    async fn upload_set_symbols(&self, emojis: Vec<SetEmoji>);

    async fn upload_symbol_emojis(&self, emojis: Vec<SymbolEmoji>);
}
