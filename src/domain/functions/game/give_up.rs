use crate::adapters::cache::Cache;
use crate::adapters::card_store::CardStore;
use crate::adapters::image_store::ImageStore;
use crate::domain::app::App;
use crate::domain::functions::game::state;
use crate::ports::clients::GameInteraction;

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub async fn give_up_command<I: GameInteraction>(&self, interaction: &I) {
        let Some(game_state) = state::fetch(interaction.id(), &self.cache).await else {
            return;
        };

        state::delete(interaction.id(), &self.cache).await;

        let Ok(images) = self.image_store.fetch(game_state.card()).await else {
            log::warn!("couldn't fetch image");
            return;
        };

        if let Err(why) = interaction.game_failed_message(game_state, images).await {
            log::warn!("couldn't send game failed: {}", why);
        }
    }
}
