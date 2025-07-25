use crate::game::state::GameState;
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

#[async_trait]
pub trait GameInteraction {
    async fn send_card(
        &self,
        card: FuzzyFound,
        images: Images,
    ) -> Result<(), MessageInterationError>;
    async fn send_guess_wrong_message(
        &self,
        state: GameState,
        images: Images,
        guess: String,
    ) -> Result<(), MessageInterationError>;
    async fn send_new_game_message(
        &self,
        state: GameState,
        images: Images,
    ) -> Result<(), MessageInterationError>;
    async fn send_win_message(
        &self,
        state: GameState,
        images: Images,
    ) -> Result<(), MessageInterationError>;
    async fn game_failed_message(
        &self,
        state: GameState,
        images: Images,
    ) -> Result<(), MessageInterationError>;
    fn id(&self) -> String;
    async fn reply(&self, message: String) -> Result<(), MessageInterationError>;
}
