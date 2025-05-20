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

    pub fn multiplier(&self) -> usize {
        match self.difficulty {
            Difficulty::Hard => 3,
            Difficulty::Medium => 2,
            Difficulty::Easy => 1,
        }
    }

    pub fn to_embed(&self) -> CreateEmbed {
        self.card
            .to_game_embed(self.multiplier(), self.guess_number)
    }

    pub fn to_full_embed(self) -> CreateEmbed {
        let (embed, _) = self.card.to_embed();
        embed
    }

    pub fn card(&self) -> &FuzzyFound {
        &self.card
    }

    pub fn number_of_guesses(&self) -> usize {
        self.guess_number
    }

    pub fn add_guess(&mut self) {
        self.guess_number += 1;
    }
}
