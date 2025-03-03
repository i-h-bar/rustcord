pub mod colours;
pub(crate) mod fuzzy;

use log;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use serenity::all::{Context, CreateMessage, Message};
use unicode_normalization::UnicodeNormalization;

const CARD_QUERY_RE: &str = r#"(?i)\[\[(.*?)(:?(?:\s)?\|(?:\s)?(:?set(?:\s)?=(?:\s)?(.*?)?)?)?(:?(?:\s)?\|(?:\s)?(:?artist(?:\s)?=(?:\s)?(.*?)?)?)?]]"#;
const SYMBOL_RE: &str = r#"(\{T}|\{Q}|\{E}|\{P}|\{PW}|\{CHAOS}|\{A}|\{TK}|\{X}|\{Y}|\{Z}|\{0}|\{Â½}|\{1}|\{2}|\{3}|\{4}|\{5}|\{6}|\{7}|\{8}|\{9}|\{10}|\{11}|\{12}|\{13}|\{14}|\{15}|\{16}|\{17}|\{18}|\{19}|\{20}|\{100}|\{1000000}|\{âˆž}|\{W/U}|\{W/B}|\{B/R}|\{B/G}|\{U/B}|\{U/R}|\{R/G}|\{R/W}|\{G/W}|\{G/U}|\{B/G/P}|\{B/R/P}|\{G/U/P}|\{G/W/P}|\{R/G/P}|\{R/W/P}|\{U/B/P}|\{U/R/P}|\{W/B/P}|\{W/U/P}|\{C/W}|\{C/U}|\{C/B}|\{C/R}|\{C/G}|\{2/W}|\{2/U}|\{2/B}|\{2/R}|\{2/G}|\{H}|\{W/P}|\{U/P}|\{B/P}|\{R/P}|\{G/P}|\{C/P}|\{HW}|\{HR}|\{W}|\{U}|\{B}|\{R}|\{G}|\{C}|\{S}|\{L}|\{D})"#;
const REMINDER_TEXT: &str = r#"\((.+)\)"#;

pub static REGEX_COLLECTION: Lazy<RegexCollection> = Lazy::new(|| {
    let punctuation_removal = Regex::new(r#"[^\w\s]"#).expect("Invalid regex");
    let cards = Regex::new(CARD_QUERY_RE).expect("Invalid regex");
    let symbols = Regex::new(SYMBOL_RE).expect("Invalid regex");
    let reminder_text = Regex::new(REMINDER_TEXT).expect("Invalid regex");
    RegexCollection {
        punctuation_removal,
        cards,
        symbols,
        reminder_text,
    }
});

pub struct RegexCollection {
    pub punctuation_removal: Regex,
    pub cards: Regex,
    pub symbols: Regex,
    pub reminder_text: Regex,
}

pub async fn send_message(message: CreateMessage, msg: &Message, ctx: &Context) {
    match msg.channel_id.send_message(&ctx.http, message).await {
        Err(why) => {
            log::warn!("Error sending message to {why:?}")
        }
        Ok(response) => {
            log::info!("Sent message to {:?}", response.channel_id)
        }
    }
}

pub async fn send(content: &str, msg: &Message, ctx: &Context) {
    match msg.channel_id.say(&ctx.http, content).await {
        Err(why) => {
            log::warn!("Error sending message - {why:?}")
        }
        Ok(_) => {
            log::info!("Sent message")
        }
    }
}

pub fn normalise(name: &str) -> String {
    REGEX_COLLECTION
        .punctuation_removal
        .replace(&name.replace("-", " ").nfkc().collect::<String>(), "")
        .to_lowercase()
}

pub fn italicise_reminder_text(text: &str) -> String {
    REGEX_COLLECTION
        .reminder_text
        .replace_all(text, |cap: &Captures| format!("(*{}*)", &cap[1]))
        .to_string()
}
