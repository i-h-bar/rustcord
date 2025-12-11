use crate::domain::app::App;
use crate::domain::functions::game::state;
use crate::domain::utils::mutex;
use crate::domain::utils::{fuzzy, normalise};
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
                log::warn!("couldn't create interaction: {why}");
            }
            return;
        };
        game_state.add_guess();

        if fuzzy::jaro_winkler_ascii_bitmask(
            &normalise(&guess),
            &game_state.card().front_normalised_name,
        ) > 0.75
        {
            let Ok(images) = self.image_store.fetch(game_state.card()).await else {
                log::warn!("couldn't fetch image");
                return;
            };

            if let Err(why) = interaction.send_win_message(game_state, images).await {
                log::warn!("couldn't send win message: {why}");
            }

            state::delete(interaction.id(), &self.cache).await;
        } else if game_state.number_of_guesses() >= game_state.max_guesses() {
            let Ok(images) = self.image_store.fetch(game_state.card()).await else {
                log::warn!("couldn't fetch image");
                return;
            };

            state::delete(interaction.id(), &self.cache).await;
            if let Err(why) = interaction.game_failed_message(game_state, images).await {
                log::warn!("couldn't send game failed message: {why}");
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
                log::warn!("couldn't send guess wrong message: {why}");
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

#[cfg(test)]
mod tests {
    use super::*;
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
    async fn test_guess_correct_wins_game() {
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
            .expect_send_win_message()
            .times(1)
            .withf(|state: &GameState, imgs: &Images| {
                state.number_of_guesses() == 1 && imgs.front == vec![1, 2, 3, 4]
            })
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);
        let options = GuessOptions::new("Lightning Bolt".to_string());

        app.guess_command(&interaction, options).await;
    }

    #[tokio::test]
    async fn test_guess_with_typo_still_wins() {
        let card = create_test_card();
        let game_state = GameState::from(card.clone(), Difficulty::Easy);
        let channel_id = "test_channel_typo".to_string();
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
            .expect_send_win_message()
            .times(1)
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);
        // "Lightningg Boltt" should match due to fuzzy matching
        let options = GuessOptions::new("Lightningg Boltt".to_string());

        app.guess_command(&interaction, options).await;
    }

    #[tokio::test]
    async fn test_guess_incorrect_continues_game() {
        let card = create_test_card();
        let mut game_state = GameState::from(card.clone(), Difficulty::Medium);
        let channel_id = "test_channel_wrong".to_string();
        let images = create_test_images();

        let ron_string = ron::to_string(&game_state).unwrap();

        let mut cache = MockCache::new();
        cache
            .expect_get()
            .times(1)
            .with(eq(channel_id.clone()))
            .return_const(Some(ron_string));
        game_state.add_guess();
        let updated_ron = ron::to_string(&game_state).unwrap();
        cache
            .expect_set()
            .times(1)
            .with(eq(channel_id.clone()), eq(updated_ron))
            .returning(|_, _| Ok(()));

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch_illustration()
            .times(1)
            .return_const(Ok(images.clone()));

        let card_store = MockCardStore::new();

        let mut interaction = MockGameInteraction::new();
        interaction.expect_id().return_const(channel_id.clone());
        interaction
            .expect_send_guess_wrong_message()
            .times(1)
            .withf(|state: &GameState, _imgs: &Images, guess: &String| {
                state.number_of_guesses() == 1 && guess == "Shock"
            })
            .returning(|_, _, _| Ok(()));

        let app = App::new(image_store, card_store, cache);
        let options = GuessOptions::new("Shock".to_string());

        app.guess_command(&interaction, options).await;
    }

    #[tokio::test]
    async fn test_guess_max_guesses_fails_game() {
        let card = create_test_card();
        let mut game_state = GameState::from(card.clone(), Difficulty::Hard);
        // Add 3 guesses so the next one will be the 4th (max for Hard)
        for _ in 0..3 {
            game_state.add_guess();
        }
        let channel_id = "test_channel_max".to_string();
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
            .withf(|state: &GameState, _imgs: &Images| state.number_of_guesses() == 4)
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);
        let options = GuessOptions::new("Shock".to_string());

        app.guess_command(&interaction, options).await;
    }

    #[tokio::test]
    async fn test_guess_no_game_found() {
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
        interaction
            .expect_reply()
            .times(1)
            .with(eq(String::from("No game found in this channel :(")))
            .returning(|_| Ok(()));

        let app = App::new(image_store, card_store, cache);
        let options = GuessOptions::new("Lightning Bolt".to_string());

        app.guess_command(&interaction, options).await;
    }

    #[tokio::test]
    async fn test_guess_threshold_exactly_075_wins() {
        let card = create_test_card();
        let game_state = GameState::from(card.clone(), Difficulty::Easy);
        let channel_id = "test_channel_threshold".to_string();
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
            .expect_send_win_message()
            .times(1)
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);
        // "lightning bol" has Jaro-Winkler score just above 0.75 with "lightning bolt"
        let options = GuessOptions::new("lightning bol".to_string());

        app.guess_command(&interaction, options).await;
    }

    #[tokio::test]
    async fn test_guess_case_insensitive() {
        let card = create_test_card();
        let game_state = GameState::from(card.clone(), Difficulty::Medium);
        let channel_id = "test_channel_case".to_string();
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
            .expect_send_win_message()
            .times(1)
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);
        let options = GuessOptions::new("LIGHTNING BOLT".to_string());

        app.guess_command(&interaction, options).await;
    }

    #[test]
    fn test_guess_options_creation() {
        let options = GuessOptions::new("Test Guess".to_string());
        assert_eq!(options.guess, "Test Guess");
    }
}
