use crate::ports::storage::CardInfo;
use async_trait::async_trait;
use std::io::Bytes;
use uuid::Uuid;

pub struct Image(pub Uuid, pub Vec<u8>);

#[async_trait]
pub trait ImageStore {
    async fn exists(&self, card: &CardInfo) -> bool;
    async fn save(&self, image: Image);
}
