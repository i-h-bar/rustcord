use crate::ports::storage::CardInfo;
use async_trait::async_trait;
use uuid::Uuid;

pub struct Image(pub Uuid, pub Vec<u8>);
pub struct Illustration(pub Uuid, pub Vec<u8>);

#[async_trait]
pub trait ImageStore {
    async fn card_image_exists(&self, card: &CardInfo) -> bool;
    async fn card_illustration_exists(&self, card: &CardInfo) -> bool;
    async fn save_image(&self, image: Image);
    async fn save_illustration(&self, illustration: Illustration);
}
