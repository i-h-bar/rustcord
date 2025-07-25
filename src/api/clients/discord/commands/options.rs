use crate::domain::game::guess::GuessOptions;
use crate::domain::game::play::PlayOptions;
use crate::domain::game::state::Difficulty;
use crate::domain::query::QueryParams;
use crate::utils::parse::{ParseError, ResolveOption};
use serenity::all::ResolvedValue;

impl ResolveOption for PlayOptions {
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

        Ok(PlayOptions::new(set, difficulty))
    }
}

impl ResolveOption for QueryParams {
    fn resolve(options: Vec<(&str, ResolvedValue)>) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        let mut card_name = None;
        let mut set_name = None;
        let mut set_code = None;
        let mut artist = None;

        for (name, value) in options {
            match name {
                "name" => {
                    card_name = match value {
                        ResolvedValue::String(card) => Some(card.to_string()),
                        _ => return Err(ParseError::new("Name was not a string")),
                    }
                }
                "set" => {
                    let set = match value {
                        ResolvedValue::String(set) => set.to_string(),
                        _ => return Err(ParseError::new("Name was not a string")),
                    };
                    if set.chars().count() < 5 {
                        set_code = Some(set);
                    } else {
                        set_name = Some(set);
                    }
                }
                "artist" => {
                    artist = match value {
                        ResolvedValue::String(artist) => Some(artist.to_string()),
                        _ => return Err(ParseError::new("Artist was not a string")),
                    }
                }
                _ => {}
            }
        }

        let Some(name) = card_name else {
            return Err(ParseError::new("No name found in query params"));
        };

        Ok(Self::new(artist, name, set_code, set_name))
    }
}

impl ResolveOption for GuessOptions {
    fn resolve(options: Vec<(&str, ResolvedValue)>) -> Result<Self, ParseError> {
        let Some((_, guess)) = options.first() else {
            return Err(ParseError::new("Could not get first option"));
        };

        let guess = match guess {
            ResolvedValue::String(guess) => (*guess).to_string(),
            _ => return Err(ParseError::new("ResolvedValue was not a string")),
        };

        Ok(GuessOptions::new(guess))
    }
}
