use crate::image_store::Images;
use crate::mtg::card::FuzzyFound;
use async_trait::async_trait;
use thiserror::Error;

mod discord;

#[derive(Debug, Error)]
#[error("An error occurred while processing a message")]
pub struct MessageInterationError(String);

#[async_trait]
pub trait MessageInteraction {
    async fn send_card(
        &self,
        card: FuzzyFound,
        images: Images,
    ) -> Result<(), MessageInterationError>;
    async fn reply(&self, message: String) -> Result<(), MessageInterationError>;
}
