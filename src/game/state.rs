use crate::mtg::db::FuzzyFound;
use serenity::all::{CreateAttachment, CreateEmbed};

pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub struct GameState {
    card: FuzzyFound,
    difficulty: Difficulty,
    guess_number: usize,
}

impl GameState {
    pub fn from(
        card: FuzzyFound,
        difficulty: Difficulty,
    ) -> Self {
        Self {
            card,
            difficulty,
            guess_number: 0,
        }
    }

    pub fn to_embed(&self) -> CreateEmbed {
        self.card.to_initial_game_embed()
    }
}
