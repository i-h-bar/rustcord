use crate::adapters::cache::Cache;
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
    pub fn from(card: Card, difficulty: Difficulty) -> Self {
        Self {
            card,
            difficulty,
            guess_number: 0,
        }
    }

    pub fn multiplier(&self) -> usize {
        match self.difficulty {
            Difficulty::Hard => 3,
            Difficulty::Medium => 2,
            Difficulty::Easy => 1,
        }
    }

    pub fn guesses(&self) -> usize {
        self.guess_number
    }

    pub fn max_guesses(&self) -> usize {
        match self.difficulty {
            Difficulty::Hard => 4,
            Difficulty::Medium => 6,
            Difficulty::Easy => 8,
        }
    }

    pub fn difficulty(&self) -> &Difficulty {
        &self.difficulty
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

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
            log::warn!("Couldn't parse game state: {}", why);
            None
        }
    }
}

pub async fn delete<C: Cache + Send + Sync>(id: String, cache: &C) {
    if let Err(why) = cache.delete(id).await {
        log::warn!("Error deleting key from redis the response: {:?}", why);
    };
}

pub async fn add<C: Cache + Send + Sync>(game_state: &GameState, id: String, cache: &C) {
    let ron_string = match ron::to_string(&game_state) {
        Ok(ron_string) => ron_string,
        Err(err) => {
            log::warn!("Error converting game state to string: {}", err);
            return;
        }
    };

    if let Err(why) = cache.set(id, ron_string).await {
        log::warn!("Error while trying to set value in redis: {}", why);
    };
}
