use crate::ports::outbound::cache::Cache;
use crate::domain::card::Card;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Deserialize, Serialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        };

        write!(f, "{string}")
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GameState {
    pub(crate) card: Card,
    difficulty: Difficulty,
    guess_number: usize,
}

impl GameState {
    #[must_use]
    pub fn from(card: Card, difficulty: Difficulty) -> Self {
        Self {
            card,
            difficulty,
            guess_number: 0,
        }
    }

    #[must_use]
    pub fn multiplier(&self) -> usize {
        match self.difficulty {
            Difficulty::Hard => 3,
            Difficulty::Medium => 2,
            Difficulty::Easy => 1,
        }
    }

    #[must_use]
    pub fn guesses(&self) -> usize {
        self.guess_number
    }

    #[must_use]
    pub fn max_guesses(&self) -> usize {
        match self.difficulty {
            Difficulty::Hard => 4,
            Difficulty::Medium => 6,
            Difficulty::Easy => 8,
        }
    }

    #[must_use]
    pub fn difficulty(&self) -> &Difficulty {
        &self.difficulty
    }

    #[must_use]
    pub fn card(&self) -> &Card {
        &self.card
    }

    #[must_use]
    pub fn number_of_guesses(&self) -> usize {
        self.guess_number
    }

    pub fn add_guess(&mut self) {
        self.guess_number += 1;
    }
}

pub async fn fetch<C: Cache + Send + Sync>(id: String, cache: &C) -> Option<GameState> {
    let game_state_string = cache.get(id).await?;

    match ron::from_str::<GameState>(&game_state_string) {
        Ok(game_state) => Some(game_state),
        Err(why) => {
            log::warn!("Couldn't parse game state: {why}");
            None
        }
    }
}

pub async fn delete<C: Cache + Send + Sync>(id: String, cache: &C) {
    if let Err(why) = cache.delete(id).await {
        log::warn!("Error deleting key from redis the response: {why:?}");
    };
}

pub async fn add<C: Cache + Send + Sync>(game_state: &GameState, id: String, cache: &C) {
    let ron_string = match ron::to_string(&game_state) {
        Ok(ron_string) => ron_string,
        Err(err) => {
            log::warn!("Error converting game state to string: {err}");
            return;
        }
    };

    if let Err(why) = cache.set(id, ron_string).await {
        log::warn!("Error while trying to set value in redis: {why}");
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::outbound::cache::MockCache;
    use mockall::predicate::eq;
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

    #[test]
    fn test_difficulty_display() {
        assert_eq!(format!("{}", Difficulty::Easy), "Easy");
        assert_eq!(format!("{}", Difficulty::Medium), "Medium");
        assert_eq!(format!("{}", Difficulty::Hard), "Hard");
    }

    #[test]
    fn test_game_state_creation() {
        let card = create_test_card();
        let state = GameState::from(card, Difficulty::Medium);

        assert_eq!(state.number_of_guesses(), 0);
        assert_eq!(state.max_guesses(), 6);
        assert_eq!(state.multiplier(), 2);
    }

    #[test]
    fn test_game_state_easy_difficulty() {
        let card = create_test_card();
        let state = GameState::from(card, Difficulty::Easy);

        assert_eq!(state.max_guesses(), 8);
        assert_eq!(state.multiplier(), 1);
    }

    #[test]
    fn test_game_state_medium_difficulty() {
        let card = create_test_card();
        let state = GameState::from(card, Difficulty::Medium);

        assert_eq!(state.max_guesses(), 6);
        assert_eq!(state.multiplier(), 2);
    }

    #[test]
    fn test_game_state_hard_difficulty() {
        let card = create_test_card();
        let state = GameState::from(card, Difficulty::Hard);

        assert_eq!(state.max_guesses(), 4);
        assert_eq!(state.multiplier(), 3);
    }

    #[test]
    fn test_add_guess() {
        let card = create_test_card();
        let mut state = GameState::from(card, Difficulty::Medium);

        assert_eq!(state.number_of_guesses(), 0);

        state.add_guess();
        assert_eq!(state.number_of_guesses(), 1);

        state.add_guess();
        assert_eq!(state.number_of_guesses(), 2);
    }

    #[test]
    fn test_guesses_alias() {
        let card = create_test_card();
        let mut state = GameState::from(card, Difficulty::Easy);

        // Both methods should return the same value
        assert_eq!(state.guesses(), state.number_of_guesses());

        state.add_guess();
        assert_eq!(state.guesses(), state.number_of_guesses());
        assert_eq!(state.guesses(), 1);
    }

    #[test]
    fn test_game_state_at_max_guesses() {
        let card = create_test_card();
        let mut state = GameState::from(card, Difficulty::Hard);

        for _ in 0..4 {
            state.add_guess();
        }

        assert_eq!(state.number_of_guesses(), state.max_guesses());
    }

    #[test]
    fn test_game_state_beyond_max_guesses() {
        let card = create_test_card();
        let mut state = GameState::from(card, Difficulty::Hard);

        for _ in 0..10 {
            state.add_guess();
        }

        assert!(state.number_of_guesses() > state.max_guesses());
        assert_eq!(state.number_of_guesses(), 10);
    }

    #[test]
    fn test_card_access() {
        let card = create_test_card();
        let state = GameState::from(card.clone(), Difficulty::Medium);

        assert_eq!(state.card().front_name, "Lightning Bolt");
        assert_eq!(state.card().front_normalised_name, "lightning bolt");
    }

    #[test]
    fn test_difficulty_access() {
        let card = create_test_card();
        let state = GameState::from(card, Difficulty::Hard);

        match state.difficulty() {
            Difficulty::Hard => {}
            _ => panic!("Expected Hard difficulty"),
        }
    }

    #[tokio::test]
    async fn test_add_game_state_to_cache() {
        let card = create_test_card();
        let state = GameState::from(card, Difficulty::Medium);
        let channel_id = "test_channel_123".to_string();

        let mut cache = MockCache::new();
        let ron_string = ron::to_string(&state).unwrap();

        cache
            .expect_set()
            .times(1)
            .with(eq(channel_id.clone()), eq(ron_string))
            .returning(|_, _| Ok(()));

        add(&state, channel_id, &cache).await;
    }

    #[tokio::test]
    async fn test_fetch_game_state_from_cache() {
        let card = create_test_card();
        let state = GameState::from(card, Difficulty::Easy);
        let channel_id = "test_channel_456".to_string();

        let ron_string = ron::to_string(&state).unwrap();

        let mut cache = MockCache::new();
        cache
            .expect_get()
            .times(1)
            .with(eq(channel_id.clone()))
            .return_const(Some(ron_string));

        let fetched_state = fetch(channel_id, &cache).await;

        assert!(fetched_state.is_some());
        let fetched = fetched_state.unwrap();
        assert_eq!(fetched.number_of_guesses(), 0);
        assert_eq!(fetched.max_guesses(), 8);
    }

    #[tokio::test]
    async fn test_fetch_game_state_not_found() {
        let channel_id = "nonexistent_channel".to_string();

        let mut cache = MockCache::new();
        cache
            .expect_get()
            .times(1)
            .with(eq(channel_id.clone()))
            .return_const(None);

        let fetched_state = fetch(channel_id, &cache).await;
        assert!(fetched_state.is_none());
    }

    #[tokio::test]
    async fn test_fetch_game_state_invalid_ron() {
        let channel_id = "test_channel_789".to_string();
        let invalid_ron = "not valid ron data".to_string();

        let mut cache = MockCache::new();
        cache
            .expect_get()
            .times(1)
            .with(eq(channel_id.clone()))
            .return_const(Some(invalid_ron));

        let fetched_state = fetch(channel_id, &cache).await;
        assert!(fetched_state.is_none());
    }

    #[tokio::test]
    async fn test_delete_game_state() {
        let channel_id = "test_channel_delete".to_string();

        let mut cache = MockCache::new();
        cache
            .expect_delete()
            .times(1)
            .with(eq(channel_id.clone()))
            .returning(|_| Ok(()));

        delete(channel_id, &cache).await;
    }

    #[test]
    fn test_game_state_serialization() {
        let card = create_test_card();
        let state = GameState::from(card, Difficulty::Medium);

        let ron_string = ron::to_string(&state);
        assert!(ron_string.is_ok());
    }

    #[test]
    fn test_game_state_round_trip() {
        let card = create_test_card();
        let mut original_state = GameState::from(card, Difficulty::Hard);
        original_state.add_guess();
        original_state.add_guess();

        let ron_string = ron::to_string(&original_state).unwrap();
        let deserialized_state: GameState = ron::from_str(&ron_string).unwrap();

        assert_eq!(deserialized_state.number_of_guesses(), 2);
        assert_eq!(deserialized_state.max_guesses(), 4);
        assert_eq!(deserialized_state.multiplier(), 3);
        assert_eq!(
            deserialized_state.card().front_name,
            "Lightning Bolt"
        );
    }
}
