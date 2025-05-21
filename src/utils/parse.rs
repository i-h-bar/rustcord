use std::fmt;
use serenity::all::{ResolvedOption, ResolvedValue};


#[derive(Debug, Clone)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    pub fn new(message: &str) -> Self {
        Self { message: message.to_string() }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseError: {}", self.message)
    }
}

pub trait ResolveOption {
    fn resolve(options: Vec<(&str, ResolvedValue)>) -> Result<Self, ParseError> where Self: Sized;
}

pub fn options<T: ResolveOption>(options: Vec<ResolvedOption>) -> Result<T, ParseError> {
    let options: Vec<(&str, ResolvedValue)> =  options
        .into_iter()
        .map(|option| {
            (option.name, option.value.clone()) 
        })
        .collect();
    
    T::resolve(options)
}