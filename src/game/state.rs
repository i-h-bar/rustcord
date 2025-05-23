use crate::mtg::db::FuzzyFound;
use crate::dbs::redis::Redis;
use serde::{Deserialize, Serialize};
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, MessageBuilder,
};

#[derive(Debug, Deserialize, Serialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn to_string(&self) -> String {
        match self {
            Difficulty::Easy => "Easy".into(),
            Difficulty::Medium => "Medium".into(),
            Difficulty::Hard => "Hard".into(),
        }
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
            Difficulty::Hard => 8,
            Difficulty::Medium => 12,
            Difficulty::Easy => 16,
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
    let Some(redis) = Redis::instance() else {
        log::warn!("Could not get redis connection");
        return None;
    };
    let Some(game_state_string): Option<String> =
        redis.get(interaction.channel_id.to_string()).await
    else {
        let message = MessageBuilder::new()
            .mention(&interaction.user)
            .push(" no game found in ")
            .push(
                interaction
                    .channel_id
                    .name(ctx)
                    .await
                    .unwrap_or(interaction.channel_id.to_string()),
            )
            .build();
        let response = CreateInteractionResponseMessage::new().content(message);

        let response = CreateInteractionResponse::Message(response);
        if let Err(why) = interaction.create_response(&ctx.http, response).await {
            log::warn!("couldn't create interaction: {}", why);
        };
        return None;
    };

    match ron::from_str::<GameState>(&game_state_string) {
        Ok(mut game_state) => {
            game_state.add_guess();
            Some(game_state)
        }
        Err(why) => {
            log::warn!("Couldn't parse game state: {}", why);
            None
        }
    }
}

pub async fn delete(interaction: &CommandInteraction) {
    let Some(redis) = Redis::instance() else {
        log::warn!("Could not get redis connection");
        return;
    };
    if let Err(why) = redis.delete(interaction.channel_id.to_string()).await {
        log::warn!(
            "Error deleting key: '{}' from redis the response: {:?}",
            interaction.channel_id.to_string(),
            why
        );
    };
}

pub async fn add(game_state: &GameState, interaction: &CommandInteraction) {
    let Some(redis) = Redis::instance() else {
        log::warn!("Could not get redis connection");
        return;
    };
    if let Err(why) = redis
        .set(
            interaction.channel_id.to_string(),
            ron::to_string(&game_state).unwrap(),
        )
        .await
    {
        log::warn!("Error while trying to set value in redis: {}", why);
    };
}
