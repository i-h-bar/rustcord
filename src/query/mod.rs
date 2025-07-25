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
    pub fn new(artist: Option<String>, name: String, set_code: Option<String>, set_name: Option<String>) -> Self {
        Self { artist, name, set_code, set_name }
    }
    
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
