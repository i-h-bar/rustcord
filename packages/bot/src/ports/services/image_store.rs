use async_trait::async_trait;
use contracts::{card::Card, image::Image};
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

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
