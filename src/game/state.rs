use crate::dbs::redis::REDIS;
use crate::mtg::card::FuzzyFound;
use serde::{Deserialize, Serialize};
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, MessageBuilder,
};
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

        write!(f, "{}", string)
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

pub async fn fetch(ctx: &Context, interaction: &CommandInteraction) -> Option<GameState> {
    let Some(game_state_string): Option<String> =
        REDIS.get(interaction.channel_id.to_string()).await
    else {
        let name = if let Some(channel) = &interaction.channel {
            if let Some(name) = &channel.name {
                name.to_owned()
            } else {
                String::from("this channel")
            }
        } else {
            String::from("this channel")
        };

        let message = MessageBuilder::new()
            .mention(&interaction.user)
            .push(" no game found in ")
            .push(name)
            .build();
        let response = CreateInteractionResponseMessage::new()
            .content(message)
            .ephemeral(true);

        let response = CreateInteractionResponse::Message(response);
        if let Err(why) = interaction.create_response(&ctx.http, response).await {
            log::warn!("couldn't create interaction: {}", why);
        };
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

pub async fn delete(interaction: &CommandInteraction) {
    if let Err(why) = REDIS.delete(interaction.channel_id.to_string()).await {
        log::warn!(
            "Error deleting key: '{}' from redis the response: {:?}",
            interaction.channel_id.to_string(),
            why
        );
    };
}

pub async fn add(game_state: &GameState, interaction: &CommandInteraction) {
    if let Err(why) = REDIS
        .set(
            interaction.channel_id.to_string(),
            ron::to_string(&game_state).unwrap(),
        )
        .await
    {
        log::warn!("Error while trying to set value in redis: {}", why);
    };
}
