use crate::domain::utils::REGEX_COLLECTION;
use regex::Captures;

pub mod colours;
pub mod embed;
pub mod emoji;
pub mod help;
pub mod message;
pub mod parse;

pub fn italicise_reminder_text(text: &str) -> String {
    REGEX_COLLECTION
        .reminder_text
        .replace_all(text, |cap: &Captures| format!("(*{}*)", &cap[1]))
        .to_string()
}
