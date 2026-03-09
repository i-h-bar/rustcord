use serenity::all::{ResolvedOption, ResolvedValue};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Error parsing command options")]
pub struct ParseError {
    message: String,
}

impl ParseError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

pub trait ResolveOption {
    fn resolve(options: Vec<(&str, ResolvedValue)>) -> Result<Self, ParseError>
    where
        Self: Sized;
}

pub fn options<T: ResolveOption>(options: Vec<ResolvedOption>) -> Result<T, ParseError> {
    let options: Vec<(&str, ResolvedValue)> = options
        .into_iter()
        .map(|option| (option.name, option.value.clone()))
        .collect();

    T::resolve(options)
}
