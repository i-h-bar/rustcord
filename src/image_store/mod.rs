mod file_system;

use crate::image_store::file_system::FileSystem;
use crate::mtg::card::FuzzyFound;
use async_trait::async_trait;
use thiserror::Error;

pub struct Images {
    pub front: Vec<u8>,
    pub back: Option<Vec<u8>>,
}

#[derive(Debug, Error)]
#[error("Error Retrieving Image")]
pub struct ImageRetrievalError(String);

#[async_trait]
pub trait ImageStore {
    fn new() -> Self;
    async fn fetch(&self, card: &FuzzyFound) -> Result<Images, ImageRetrievalError>;
    async fn fetch_illustration(&self, card: &FuzzyFound) -> Result<Images, ImageRetrievalError>;
}

#[must_use]
pub fn init_image_store() -> impl ImageStore {
    FileSystem::new()
}
