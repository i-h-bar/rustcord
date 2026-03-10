use contracts::{search_result::SearchResultDto, image::Image};
use crate::domain::functions::game::state::GameState;
use async_trait::async_trait;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Error)]
#[error("An error occurred while processing a message")]
pub struct MessageInteractionError(String);

impl MessageInteractionError {
    #[must_use]
    pub fn new(msg: String) -> Self {
        Self(msg)
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait MessageInteraction {
    async fn send_card(&self, result: SearchResultDto) -> Result<(), MessageInteractionError>;
    async fn reply(&self, message: String) -> Result<(), MessageInteractionError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GameInteraction {
    async fn send_guess_wrong_message(
        &self,
        state: GameState,
        images: Image,
        guess: String,
    ) -> Result<(), MessageInteractionError>;
    async fn send_new_game_message(
        &self,
        state: GameState,
        images: Image,
    ) -> Result<(), MessageInteractionError>;
    async fn send_win_message(
        &self,
        state: GameState,
        images: Image,
    ) -> Result<(), MessageInteractionError>;
    async fn game_failed_message(
        &self,
        state: GameState,
        images: Image,
    ) -> Result<(), MessageInteractionError>;
    fn id(&self) -> String;
    async fn reply(&self, message: String) -> Result<(), MessageInteractionError>;
}

#[async_trait]
pub trait Client {
    async fn run(&mut self);
}
