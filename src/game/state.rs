use crate::mtg::db::FuzzyFound;
use serde::{Deserialize, Serialize};
use serenity::all::CreateEmbed;

#[derive(Debug, Deserialize, Serialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GameState {
    card: FuzzyFound,
    difficulty: Difficulty,
    guess_number: usize,
}

impl GameState {
    pub fn from(card: FuzzyFound, difficulty: Difficulty) -> Self {
        Self {
            card,
            difficulty,
            guess_number: 0,
        }
    }

    pub fn to_embed(&self) -> CreateEmbed {
        self.card.to_initial_game_embed()
    }

    pub fn to_full_embed(self) -> CreateEmbed {
        let (embed, _) = self.card.to_embed();
        embed
    }

    pub fn card(&self) -> &FuzzyFound {
        &self.card
    }
}
