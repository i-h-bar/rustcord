use crate::adapters::cache::Cache;
use crate::adapters::card_store::CardStore;
use crate::adapters::image_store::ImageStore;
use crate::domain::app::App;
use crate::domain::functions::game::state;
use crate::domain::functions::game::state::{Difficulty, GameState};
use crate::domain::utils;
use crate::ports::clients::GameInteraction;

const SET_ABBR_CHAR_LIMIT: usize = 5;

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub async fn play_command<I: GameInteraction>(&self, interaction: &I, options: PlayOptions) {
        let PlayOptions { set, difficulty } = options;
        let random_card = if let Some(set_name) = set {
            let matched_set = if set_name.chars().count() < SET_ABBR_CHAR_LIMIT {
                self.set_from_abbreviation(&set_name).await
            } else {
                self.fuzzy_match_set_name(&utils::normalise(&set_name))
                    .await
            };

            let Some(matched_set) = matched_set else {
                if let Err(why) = interaction
                    .reply(format!("Could not find set '{set_name}'"))
                    .await
                {
                    log::error!("couldn't create interaction response: {why:?}");
                }
                return;
            };
            self.card_store.random_card_from_set(&matched_set).await
        } else {
            self.card_store.random_card().await
        };

        if let Some(card) = random_card {
            let game_state = GameState::from(card, difficulty);
            state::add(&game_state, interaction.id(), &self.cache).await;

            let Ok(images) = self.image_store.fetch_illustration(game_state.card()).await else {
                log::warn!("failed to get image");
                return;
            };

            if let Err(why) = interaction.send_new_game_message(game_state, images).await {
                log::error!("couldn't send game state: {why:?}");
            };
        } else {
            log::warn!("Failed to get random card");
        }
    }
}

pub struct PlayOptions {
    set: Option<String>,
    difficulty: Difficulty,
}

impl PlayOptions {
    pub fn new(set: Option<String>, difficulty: Difficulty) -> Self {
        Self { set, difficulty }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::cache::MockCache;
    use crate::adapters::card_store::MockCardStore;
    use crate::adapters::image_store::{Images, MockImageStore};
    use crate::domain::app::App;
    use crate::domain::card::Card;
    use crate::ports::clients::MockGameInteraction;
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
    async fn test_play_random_card() {
        let card = create_test_card();
        let channel_id = "test_channel".to_string();
        let images = create_test_images();

        let mut cache = MockCache::new();
        cache.expect_set().times(1).returning(|_, _| Ok(()));

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch_illustration()
            .times(1)
            .return_const(Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        card_store
            .expect_random_card()
            .times(1)
            .return_const(Some(card.clone()));

        let mut interaction = MockGameInteraction::new();
        interaction.expect_id().return_const(channel_id.clone());
        interaction
            .expect_send_new_game_message()
            .times(1)
            .withf(|state: &GameState, _imgs: &Images| {
                state.card().front_name == "Lightning Bolt" && state.number_of_guesses() == 0
            })
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);
        let options = PlayOptions::new(None, Difficulty::Medium);

        app.play_command(&interaction, options).await;
    }

    #[tokio::test]
    async fn test_play_with_set_abbreviation() {
        let card = create_test_card();
        let channel_id = "test_channel_set".to_string();
        let images = create_test_images();

        let mut cache = MockCache::new();
        cache.expect_set().times(1).returning(|_, _| Ok(()));

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch_illustration()
            .times(1)
            .return_const(Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        card_store
            .expect_set_name_from_abbreviation()
            .times(1)
            .with(eq("LEA"))
            .return_const(Some("Limited Edition Alpha".to_string()));
        card_store
            .expect_random_card_from_set()
            .times(1)
            .with(eq("Limited Edition Alpha"))
            .return_const(Some(card.clone()));

        let mut interaction = MockGameInteraction::new();
        interaction.expect_id().return_const(channel_id.clone());
        interaction
            .expect_send_new_game_message()
            .times(1)
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);
        let options = PlayOptions::new(Some("LEA".to_string()), Difficulty::Easy);

        app.play_command(&interaction, options).await;
    }

    #[tokio::test]
    async fn test_play_with_set_full_name() {
        let card = create_test_card();
        let channel_id = "test_channel_fullname".to_string();
        let images = create_test_images();

        let mut cache = MockCache::new();
        cache.expect_set().times(1).returning(|_, _| Ok(()));

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch_illustration()
            .times(1)
            .return_const(Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        card_store
            .expect_search_for_set_name()
            .times(1)
            .with(eq("limited edition alpha"))
            .return_const(Some(vec!["Limited Edition Alpha".to_string()]));
        card_store
            .expect_random_card_from_set()
            .times(1)
            .with(eq("Limited Edition Alpha"))
            .return_const(Some(card.clone()));

        let mut interaction = MockGameInteraction::new();
        interaction.expect_id().return_const(channel_id.clone());
        interaction
            .expect_send_new_game_message()
            .times(1)
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);
        let options = PlayOptions::new(
            Some("Limited Edition Alpha".to_string()),
            Difficulty::Hard,
        );

        app.play_command(&interaction, options).await;
    }

    #[tokio::test]
    async fn test_play_set_not_found() {
        let channel_id = "test_channel_notfound".to_string();

        let cache = MockCache::new();
        let image_store = MockImageStore::new();

        let mut card_store = MockCardStore::new();
        card_store
            .expect_set_name_from_abbreviation()
            .times(1)
            .with(eq("XYZ"))
            .return_const(None);

        let mut interaction = MockGameInteraction::new();
        interaction.expect_id().return_const(channel_id.clone());
        interaction
            .expect_reply()
            .times(1)
            .with(eq(String::from("Could not find set 'XYZ'")))
            .returning(|_| Ok(()));

        let app = App::new(image_store, card_store, cache);
        let options = PlayOptions::new(Some("XYZ".to_string()), Difficulty::Medium);

        app.play_command(&interaction, options).await;
    }

    #[tokio::test]
    async fn test_play_with_different_difficulties() {
        let card = create_test_card();
        let channel_id = "test_channel_diff".to_string();
        let images = create_test_images();

        let mut cache = MockCache::new();
        cache.expect_set().times(1).returning(|_, _| Ok(()));

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch_illustration()
            .times(1)
            .return_const(Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        card_store
            .expect_random_card()
            .times(1)
            .return_const(Some(card.clone()));

        let mut interaction = MockGameInteraction::new();
        interaction.expect_id().return_const(channel_id.clone());
        interaction
            .expect_send_new_game_message()
            .times(1)
            .withf(|state: &GameState, _imgs: &Images| {
                state.max_guesses() == 4 && state.multiplier() == 3
            })
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);
        let options = PlayOptions::new(None, Difficulty::Hard);

        app.play_command(&interaction, options).await;
    }

    #[tokio::test]
    async fn test_play_set_abbreviation_boundary() {
        let card = create_test_card();
        let channel_id = "test_channel_boundary".to_string();
        let images = create_test_images();

        let mut cache = MockCache::new();
        cache.expect_set().times(1).returning(|_, _| Ok(()));

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch_illustration()
            .times(1)
            .return_const(Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        // 4 characters is treated as abbreviation (< 5 chars)
        card_store
            .expect_set_name_from_abbreviation()
            .times(1)
            .with(eq("LEA1"))
            .return_const(Some("Limited Edition Alpha".to_string()));
        card_store
            .expect_random_card_from_set()
            .times(1)
            .return_const(Some(card.clone()));

        let mut interaction = MockGameInteraction::new();
        interaction.expect_id().return_const(channel_id.clone());
        interaction
            .expect_send_new_game_message()
            .times(1)
            .returning(|_, _| Ok(()));

        let app = App::new(image_store, card_store, cache);
        let options = PlayOptions::new(Some("LEA1".to_string()), Difficulty::Medium);

        app.play_command(&interaction, options).await;
    }

    #[test]
    fn test_play_options_creation() {
        let options = PlayOptions::new(Some("LEA".to_string()), Difficulty::Easy);
        assert_eq!(options.set, Some("LEA".to_string()));
    }

    #[test]
    fn test_play_options_no_set() {
        let options = PlayOptions::new(None, Difficulty::Medium);
        assert_eq!(options.set, None);
    }
}
