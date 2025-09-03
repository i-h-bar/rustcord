mod file_system;

use crate::adapters::image_store::file_system::FileSystem;
use crate::domain::card::Card;
use async_trait::async_trait;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, derive(Clone, PartialEq))]
#[derive(Debug)]
pub struct Images {
    pub front: Vec<u8>,
    pub back: Option<Vec<u8>>,
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Error)]
#[error("Error Retrieving Image")]
pub struct ImageRetrievalError(String);

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ImageStore {
    fn create() -> Self;
    async fn fetch(&self, card: &Card) -> Result<Images, ImageRetrievalError>;
    async fn fetch_illustration(&self, card: &Card) -> Result<Images, ImageRetrievalError>;
}

#[must_use]
pub async fn init_image_store() -> impl ImageStore {
    FileSystem::create()
}
