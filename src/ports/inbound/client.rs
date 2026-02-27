use crate::domain::card::Card;
use crate::domain::set::Set;
use crate::domain::functions::game::state::GameState;
use crate::ports::outbound::image_store::Images;
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
    async fn send_card(&self, card: Card, images: Images, sets: Option<Vec<Set>>) -> Result<(), MessageInteractionError>;
    async fn reply(&self, message: String) -> Result<(), MessageInteractionError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GameInteraction {
    async fn send_guess_wrong_message(
        &self,
        state: GameState,
        images: Images,
        guess: String,
    ) -> Result<(), MessageInteractionError>;
    async fn send_new_game_message(
        &self,
        state: GameState,
        images: Images,
    ) -> Result<(), MessageInteractionError>;
    async fn send_win_message(
        &self,
        state: GameState,
        images: Images,
    ) -> Result<(), MessageInteractionError>;
    async fn game_failed_message(
        &self,
        state: GameState,
        images: Images,
    ) -> Result<(), MessageInteractionError>;
    fn id(&self) -> String;
    async fn reply(&self, message: String) -> Result<(), MessageInteractionError>;
}

#[async_trait]
pub trait Client {
    async fn run(&mut self);
}
