use async_trait::async_trait;
use contracts::card::Card;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[derive(Debug, Error)]
#[error("Error retrieving card image: {0}")]
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
    async fn fetch(&self, card: &Card) -> Result<Vec<u8>, ImageRetrievalError>;
}
