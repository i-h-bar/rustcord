use crate::app::App;
use crate::cache::Cache;
use crate::card_store::CardStore;
use crate::clients::GameInteraction;
use crate::game::state;
use crate::game::state::{Difficulty, GameState};
use crate::image_store::ImageStore;
use crate::utils;
use crate::utils::parse::ResolveOption;
use serenity::all::{
    CommandOptionType, CreateAttachment, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, MessageBuilder, ResolvedValue,
};

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub async fn play_command<I: GameInteraction>(&self, interaction: &I, options: PlayOptions) {
        let PlayOptions { set, difficulty } = options;
        let random_card = if let Some(set_name) = set {
            let matched_set = if set_name.chars().count() < 5 {
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
                    log::error!("couldn't create interaction response: {:?}", why);
                };
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
                log::error!("couldn't send game state: {:?}", why);
            };
        } else {
            log::warn!("Failed to get random card");
        }
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("play")
        .description("Start a guess the card game")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "set",
                "What set to choose the card from",
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "difficulty",
                "what difficulty do you want to play at?",
            )
            .add_string_choice("Easy", "Easy")
            .add_string_choice("Medium", "Medium")
            .add_string_choice("Hard", "Hard")
            .required(false),
        )
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
