use crate::api::clients::GameInteraction;
use crate::domain::app::App;
use crate::domain::game::state;
use crate::spi::cache::Cache;
use crate::spi::card_store::CardStore;
use crate::spi::image_store::ImageStore;
use crate::utils::mutex;
use crate::utils::{fuzzy, normalise};

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub async fn guess_command<I: GameInteraction>(&self, interaction: &I, options: GuessOptions) {
        let channel_id = interaction.id();
        let lock = mutex::LOCKS.get(&channel_id).await;
        let _guard = lock.lock().await;
        self.run_guess(interaction, options).await;
    }

    async fn run_guess<I: GameInteraction>(&self, interaction: &I, options: GuessOptions) {
        let GuessOptions { guess } = options;

        let Some(mut game_state) = state::fetch(interaction.id(), &self.cache).await else {
            if let Err(why) = interaction
                .reply(String::from("No game found in this channel :("))
                .await
            {
                log::warn!("couldn't create interaction: {}", why);
            }
            return;
        };
        game_state.add_guess();

        if fuzzy::jaro_winkler(&normalise(&guess), &game_state.card().front_normalised_name) > 0.75
        {
            let Ok(images) = self.image_store.fetch(game_state.card()).await else {
                log::warn!("couldn't fetch image");
                return;
            };

            if let Err(why) = interaction.send_win_message(game_state, images).await {
                log::warn!("couldn't send win message: {}", why);
            }

            state::delete(interaction.id(), &self.cache).await;
        } else if game_state.number_of_guesses() >= game_state.max_guesses() {
            let Ok(images) = self.image_store.fetch(game_state.card()).await else {
                log::warn!("couldn't fetch image");
                return;
            };

            state::delete(interaction.id(), &self.cache).await;
            if let Err(why) = interaction.game_failed_message(game_state, images).await {
                log::warn!("couldn't send game failed message: {}", why);
            }
        } else {
            state::add(&game_state, interaction.id(), &self.cache).await;
            let Ok(images) = self.image_store.fetch_illustration(game_state.card()).await else {
                log::warn!("couldn't fetch illustration");
                return;
            };
            if let Err(why) = interaction
                .send_guess_wrong_message(game_state, images, guess)
                .await
            {
                log::warn!("couldn't send guess wrong message: {}", why);
            }
        }
    }
}

pub struct GuessOptions {
    guess: String,
}

impl GuessOptions {
    pub fn new(guess: String) -> Self {
        Self { guess }
    }
}
