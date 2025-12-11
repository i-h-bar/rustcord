use crate::domain::app::App;
use crate::domain::functions::game::state;
use crate::ports::inbound::client::GameInteraction;
use crate::ports::outbound::cache::Cache;
use crate::ports::outbound::card_store::CardStore;
use crate::ports::outbound::image_store::ImageStore;

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
            log::warn!("couldn't send game failed: {why}");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::app::App;
    use crate::domain::card::Card;
    use crate::domain::functions::game::state::{Difficulty, GameState};
    use crate::ports::inbound::client::MockGameInteraction;
    use crate::ports::outbound::cache::MockCache;
    use crate::ports::outbound::card_store::MockCardStore;
    use crate::ports::outbound::image_store::{Images, MockImageStore};
    use mockall::predicate::*;
    use uuid::uuid;

    fn create_test_card() -> Card {
        Card {
            front_name: "Lightning Bolt".to_string(),
            front_normalised_name: "lightning bolt".to_string(),
            front_scryfall_url: "https://scryfall.com/card/test".to_string(),
            front_image_id: uuid!("12345678-1234-1234-1234-123456789012"),
            front_illustration_id: Some(uuid!("12345678-1234-1234-1234-123456789013")),
            front_mana_cost: "{R}".to_string(),
            front_colour_identity: vec!["R".to_string()],
            front_power: None,
            front_toughness: None,
            front_loyalty: None,
            front_defence: None,
            front_type_line: "Instant".to_string(),
            front_oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
            back_name: None,
            back_scryfall_url: None,
            back_image_id: None,
            back_illustration_id: None,
            back_mana_cost: None,
            back_colour_identity: None,
            back_power: None,
            back_toughness: None,
            back_loyalty: None,
            back_defence: None,
            back_type_line: None,
            back_oracle_text: None,
            artist: "Christopher Rush".to_string(),
            set_name: "Limited Edition Alpha".to_string(),
        }
    }

    fn create_test_images() -> Images {
        Images {
            front: vec![1, 2, 3, 4],
            back: None,
        }
    }

    #[tokio::test]
    async fn test_give_up_ends_game_successfully() {
        let card = create_test_card();
        let game_state = GameState::from(card.clone(), Difficulty::Medium);
        let channel_id = "test_channel".to_string();
        let images = create_test_images();

        let ron_string = ron::to_string(&game_state).unwrap();

        let mut cache = MockCache::new();
        cache
            .expect_get()
            .times(1)
            .with(eq(channel_id.clone()))
            .return_const(Some(ron_string));
        cache
            .expect_delete()
            .times(1)
            .with(eq(channel_id.clone()))
            .returning(|_| Ok(()));

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch()
            .times(1)
            .with(eq(card.clone()))
            .return_const(Ok(images.clone()));

        let card_store = MockCardStore::new();

        let mut interaction = MockGameInteraction::new();
        interaction.expect_id().return_const(channel_id.clone());
        interaction
            .expect_game_failed_message()
            .times(1)
            .withf(|state: &GameState, imgs: &Images| {
                state.card().front_name == "Lightning Bolt" && imgs.front == vec![1, 2, 3, 4]
            })
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);

        app.give_up_command(&interaction).await;
    }

    #[tokio::test]
    async fn test_give_up_with_multiple_guesses() {
        let card = create_test_card();
        let mut game_state = GameState::from(card.clone(), Difficulty::Hard);
        // Simulate player made some guesses before giving up
        game_state.add_guess();
        game_state.add_guess();
        let channel_id = "test_channel_guesses".to_string();
        let images = create_test_images();

        let ron_string = ron::to_string(&game_state).unwrap();

        let mut cache = MockCache::new();
        cache
            .expect_get()
            .times(1)
            .with(eq(channel_id.clone()))
            .return_const(Some(ron_string));
        cache
            .expect_delete()
            .times(1)
            .with(eq(channel_id.clone()))
            .returning(|_| Ok(()));

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch()
            .times(1)
            .return_const(Ok(images.clone()));

        let card_store = MockCardStore::new();

        let mut interaction = MockGameInteraction::new();
        interaction.expect_id().return_const(channel_id.clone());
        interaction
            .expect_game_failed_message()
            .times(1)
            .withf(|state: &GameState, _imgs: &Images| state.number_of_guesses() == 2)
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);

        app.give_up_command(&interaction).await;
    }

    #[tokio::test]
    async fn test_give_up_no_game_found() {
        let channel_id = "test_channel_none".to_string();

        let mut cache = MockCache::new();
        cache
            .expect_get()
            .times(1)
            .with(eq(channel_id.clone()))
            .return_const(None);

        let image_store = MockImageStore::new();
        let card_store = MockCardStore::new();

        let mut interaction = MockGameInteraction::new();
        interaction.expect_id().return_const(channel_id.clone());
        // Should return early without calling any other methods

        let app = App::new(image_store, card_store, cache);

        app.give_up_command(&interaction).await;
    }

    #[tokio::test]
    async fn test_give_up_deletes_game_state() {
        let card = create_test_card();
        let game_state = GameState::from(card.clone(), Difficulty::Easy);
        let channel_id = "test_channel_delete".to_string();
        let images = create_test_images();

        let ron_string = ron::to_string(&game_state).unwrap();

        let mut cache = MockCache::new();
        cache
            .expect_get()
            .times(1)
            .with(eq(channel_id.clone()))
            .return_const(Some(ron_string));
        // This is the key assertion - delete must be called
        cache
            .expect_delete()
            .times(1)
            .with(eq(channel_id.clone()))
            .returning(|_| Ok(()));

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch()
            .times(1)
            .return_const(Ok(images.clone()));

        let card_store = MockCardStore::new();

        let mut interaction = MockGameInteraction::new();
        interaction.expect_id().return_const(channel_id.clone());
        interaction
            .expect_game_failed_message()
            .times(1)
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);

        app.give_up_command(&interaction).await;
    }

    #[tokio::test]
    async fn test_give_up_with_all_difficulty_levels() {
        for difficulty in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
            let card = create_test_card();
            let game_state = GameState::from(card.clone(), difficulty);
            let channel_id = format!("test_channel_{:?}", game_state.difficulty());
            let images = create_test_images();

            let ron_string = ron::to_string(&game_state).unwrap();

            let mut cache = MockCache::new();
            cache.expect_get().times(1).return_const(Some(ron_string));
            cache.expect_delete().times(1).returning(|_| Ok(()));

            let mut image_store = MockImageStore::new();
            image_store
                .expect_fetch()
                .times(1)
                .return_const(Ok(images.clone()));

            let card_store = MockCardStore::new();

            let mut interaction = MockGameInteraction::new();
            interaction.expect_id().return_const(channel_id.clone());
            interaction
                .expect_game_failed_message()
                .times(1)
                .returning(|_, _| Ok(()));

            let app = App::new(image_store, card_store, cache);

            app.give_up_command(&interaction).await;
        }
    }
}
