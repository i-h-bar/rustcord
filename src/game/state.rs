use crate::cache::Cache;
use crate::mtg::card::FuzzyFound;
use serde::{Deserialize, Serialize};
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, MessageBuilder,
};
use std::fmt::{Display, Formatter};
use crate::clients::MessageInteraction;

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

    pub fn to_embed(&self) -> CreateEmbed {
        self.card
            .to_game_embed(self.multiplier(), self.guess_number)
    }

    pub fn convert_to_embed(self) -> CreateEmbed {
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

pub async fn fetch<C: Cache + Send + Sync, I: MessageInteraction>(
    interaction: &I,
    cache: &C,
) -> Option<GameState> {
    let Some(game_state_string): Option<String> =
        cache.get(interaction.id()).await
    else {
        if let Err(why) = interaction.reply(String::from("No game found in this channel :(")).await {
            log::warn!("couldn't create interaction: {}", why);
        }
        return None;
    };

    match ron::from_str::<GameState>(&game_state_string) {
        Ok(game_state) => Some(game_state),
        Err(why) => {
            log::warn!("Couldn't parse game state: {}", why);
            None
        }
    }
}

pub async fn delete<C: Cache + Send + Sync, I: MessageInteraction>(interaction: &I, cache: &C) {
    if let Err(why) = cache.delete(interaction.id()).await {
        log::warn!(
            "Error deleting key: '{}' from redis the response: {:?}",
            interaction.id(),
            why
        );
    };
}

pub async fn add<C: Cache + Send + Sync, I: MessageInteraction>(
    game_state: &GameState,
    interaction: &I,
    cache: &C,
) {
    let ron_string = match ron::to_string(&game_state) {
        Ok(ron_string) => ron_string,
        Err(err) => {
            log::warn!("Error converting game state to string: {}", err);
            return;
        }
    };

    if let Err(why) = cache
        .set(interaction.id(), ron_string)
        .await
    {
        log::warn!("Error while trying to set value in redis: {}", why);
    };
}
