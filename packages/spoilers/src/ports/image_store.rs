use std::io::Bytes;
use async_trait::async_trait;
use uuid::Uuid;
use crate::ports::storage::CardInfo;


pub struct Image(pub Uuid, pub Vec<u8>);

#[async_trait]
pub trait ImageStore {
    async fn exists(&self, card: &CardInfo) -> bool;
    async fn save(&self, image: Image);
}