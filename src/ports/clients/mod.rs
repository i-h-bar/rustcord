use crate::adapters::cache::Cache;
use crate::adapters::card_store::CardStore;
use crate::adapters::image_store::{ImageStore, Images};
use crate::domain::app::App;
use crate::domain::card::Card;
use crate::domain::functions::game::state::GameState;
use crate::ports::clients::discord::client::Discord;
use async_trait::async_trait;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

mod discord;

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Error)]
#[error("An error occurred while processing a message")]
pub struct MessageInterationError(String);

#[cfg_attr(test, automock)]
#[async_trait]
pub trait MessageInteraction {
    async fn send_card(&self, card: Card, images: Images) -> Result<(), MessageInterationError>;
    async fn reply(&self, message: String) -> Result<(), MessageInterationError>;
}

#[async_trait]
pub trait GameInteraction {
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

#[async_trait]
pub trait Client {
    async fn run(&mut self);
}

pub async fn create_client<IS, CS, C>(app: App<IS, CS, C>) -> impl Client
where
    IS: ImageStore + Send + Sync + 'static,
    CS: CardStore + Send + Sync + 'static,
    C: Cache + Send + Sync + 'static,
{
    Discord::new(app).await
}
