use crate::domain::dto::card::Card;
use async_trait::async_trait;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, derive(Clone, PartialEq))]
#[derive(Debug)]
pub struct Image {
    bytes: Vec<u8>,
}

impl Image {
    pub fn new(image: Vec<u8>) -> Self {
        Self { bytes: image }
    }
    
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Error)]
#[error("Error Retrieving Image")]
pub struct ImageRetrievalError(String);

impl ImageRetrievalError {
    #[must_use]
    pub fn new(msg: String) -> Self {
        Self(msg)
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ImageStore {
    fn create() -> Self;
    async fn fetch(&self, card: &Card) -> Result<Image, ImageRetrievalError>;
    async fn fetch_illustration(&self, card: &Card) -> Result<Image, ImageRetrievalError>;
}
