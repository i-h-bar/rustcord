use crate::utils;
use crate::utils::parse::{ParseError, ResolveOption};
use regex::Captures;
use serenity::all::ResolvedValue;

pub struct QueryParams {
    artist: Option<String>,
    name: String,
    set_code: Option<String>,
    set_name: Option<String>,
}

impl QueryParams {
    #[must_use]
    pub fn from(capture: &Captures<'_>) -> Option<Self> {
        let raw_name = capture.get(1)?.as_str();
        let name = utils::normalise(raw_name);
        let (set_code, set_name) = match capture.get(4) {
            Some(set) => {
                let set = set.as_str();
                if set.chars().count() < 5 {
                    (Some(utils::normalise(set)), None)
                } else {
                    (None, Some(utils::normalise(set)))
                }
            }
            None => (None, None),
        };

        let artist = capture
            .get(7)
            .map(|artist| utils::normalise(artist.as_str()));

        Some(Self {
            artist,
            name,
            set_code,
            set_name,
        })
    }

    #[must_use]
    pub fn set_code(&self) -> Option<&String> {
        self.set_code.as_ref()
    }

    #[must_use]
    pub fn set_name(&self) -> Option<&String> {
        self.set_name.as_ref()
    }

    #[must_use]
    pub fn artist(&self) -> Option<&String> {
        self.artist.as_ref()
    }

    #[must_use]
    pub fn name(&self) -> &String {
        &self.name
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

        Ok(Self {
            artist,
            name,
            set_code,
            set_name,
        })
    }
}
