use async_trait::async_trait;
use thiserror::Error;
use crate::mtg::card::FuzzyFound;


pub struct Images {
    front: Vec<u8>,
    back: Option<Vec<u8>>,
}


#[derive(Debug, Error)]
#[error("Error Retrieving Image")]
pub struct ImageRetrievalError;

#[async_trait]
pub trait ImageStore {
    fn new() -> Self;
    async fn fetch(&self, card: &FuzzyFound) -> Result<Images, ImageRetrievalError>;
    async fn fetch_illustration(&self, card: &FuzzyFound) -> Result<Images, ImageRetrievalError>;
}