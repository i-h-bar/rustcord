pub mod colours;
pub(crate) mod fuzzy;

use crate::mtg::db::FuzzyFound;
use log;
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::all::{
    Context, CreateAttachment, CreateEmbed, CreateMessage, Embed, Message, MessageBuilder,
};
use serenity::model::Colour;
use unicode_normalization::UnicodeNormalization;

const CARD_QUERY_RE: &str = r#"(?i)\[\[(.*?)(:?(?:\s)?\|(?:\s)?(:?set(?:\s)?=(?:\s)?(.*?)?)?)?(:?(?:\s)?\|(?:\s)?(:?artist(?:\s)?=(?:\s)?(.*?)?)?)?]]"#;

pub static REGEX_COLLECTION: Lazy<RegexCollection> = Lazy::new(|| {
    let punctuation_removal = Regex::new(r#"[^\w\s]"#).expect("Invalid regex");
    let cards = Regex::new(CARD_QUERY_RE).expect("Invalid regex");
    RegexCollection {
        punctuation_removal,
        cards,
    }
});

pub struct RegexCollection {
    pub punctuation_removal: Regex,
    pub cards: Regex,
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

pub async fn send_image(
    image: &Vec<u8>,
    image_name: &String,
    content: Option<&str>,
    msg: &Message,
    ctx: &Context,
) {
    let message = if let Some(content) = content {
        let message = CreateMessage::new();
        message.content(content)
    } else {
        CreateMessage::new()
    };

    let attachment = CreateAttachment::bytes(image.to_vec(), image_name);
    let message = message.add_file(attachment);

    match msg.channel_id.send_message(&ctx.http, message).await {
        Err(why) => {
            log::warn!("Error sending image - {why:?}")
        }
        Ok(_) => {
            log::info!("Sent '{}' image", image_name)
        }
    }
}

pub fn normalise(name: &str) -> String {
    REGEX_COLLECTION
        .punctuation_removal
        .replace(&name.replace("-", " ").nfkc().collect::<String>(), "")
        .to_lowercase()
}
