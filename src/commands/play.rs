use crate::app::App;
use crate::cache::Cache;
use crate::card_store::CardStore;
use crate::game::state;
use crate::game::state::{Difficulty, GameState};
use crate::image_store::ImageStore;
use crate::utils;
use crate::utils::parse;
use crate::utils::parse::{ParseError, ResolveOption};
use serenity::all::{
    CommandInteraction, CommandOptionType, CreateAttachment, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, MessageBuilder, ResolvedValue,
};
use serenity::prelude::*;

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub async fn play_command(&self, ctx: &Context, interaction: &CommandInteraction) {
        let Options { set, difficulty } = match parse::options(interaction.data.options()) {
            Ok(options) => options,
            Err(err) => {
                log::warn!("{}", err);
                return;
            }
        };

        let random_card = if let Some(set_name) = set {
            let matched_set = if set_name.chars().count() < 5 {
                self.set_from_abbreviation(&set_name).await
            } else {
                self.fuzzy_match_set_name(&utils::normalise(&set_name))
                    .await
            };

            let Some(matched_set) = matched_set else {
                let message = MessageBuilder::new()
                    .mention(&interaction.user)
                    .push(" could not find set named ")
                    .push(set_name)
                    .build();

                let response = CreateInteractionResponseMessage::new().content(message);
                if let Err(why) = interaction
                    .create_response(ctx, CreateInteractionResponse::Message(response))
                    .await
                {
                    log::error!("couldn't create interaction response: {:?}", why);
                }
                return;
            };
            self.card_store.random_card_from_set(&matched_set).await
        } else {
            self.card_store.random_card().await
        };

        if let Some(card) = random_card {
            let game_state = GameState::from(card, difficulty);
            let Ok(images) = self.image_store.fetch_illustration(game_state.card()).await else {
                log::warn!("failed to get image");
                return;
            };

            let Some(illustration_id) = game_state.card().front_illustration_id() else {
                log::warn!("failed to get image id");
                return;
            };

            let illustration =
                CreateAttachment::bytes(images.front, format!("{illustration_id}.png",));

            let response = match game_state.difficulty() {
                Difficulty::Hard => CreateInteractionResponseMessage::new().content(format!(
                    "Difficulty is set to `{}`.",
                    game_state.difficulty()
                )),
                _ => CreateInteractionResponseMessage::new().content(format!(
                    "Difficulty is set to `{}`. This card is from `{}`",
                    game_state.difficulty(),
                    game_state.card().set_name()
                )),
            }
            .add_file(illustration)
            .add_embed(game_state.to_embed());

            let response = CreateInteractionResponse::Message(response);
            if let Err(why) = interaction.create_response(&ctx.http, response).await {
                log::error!("couldn't create interaction response: {:?}", why);
            }

            state::add(&game_state, interaction, &self.cache).await;
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

struct Options {
    set: Option<String>,
    difficulty: Difficulty,
}

impl ResolveOption for Options {
    fn resolve(option: Vec<(&str, ResolvedValue)>) -> Result<Self, ParseError> {
        let mut set: Option<String> = None;
        let mut difficulty: Difficulty = Difficulty::Medium;

        for (name, value) in option {
            match name {
                "set" => {
                    set = match value {
                        ResolvedValue::String(card) => Some(card.to_string()),
                        _ => return Err(ParseError::new("set ResolvedValue was not a string")),
                    };
                }
                "difficulty" => {
                    difficulty = match value {
                        ResolvedValue::String(difficulty_string) => match difficulty_string {
                            "Easy" => Difficulty::Easy,
                            "Medium" => Difficulty::Medium,
                            "Hard" => Difficulty::Hard,
                            default => {
                                return Err(ParseError::new(&format!(
                                    "Could not parse {default} into difficulty"
                                )))
                            }
                        },
                        _ => {
                            return Err(ParseError::new(
                                "difficulty ResolvedValue was not a string",
                            ))
                        }
                    };
                }
                _ => {}
            }
        }

        Ok(Options { set, difficulty })
    }
}
