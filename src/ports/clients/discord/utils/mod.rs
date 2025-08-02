use regex::Captures;
use crate::domain::utils::REGEX_COLLECTION;

pub mod embed;
pub mod parse;
pub mod help;
pub mod colours;
pub mod emoji;

pub fn italicise_reminder_text(text: &str) -> String {
    REGEX_COLLECTION
        .reminder_text
        .replace_all(text, |cap: &Captures| format!("(*{}*)", &cap[1]))
        .to_string()
}