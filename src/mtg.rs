use lazy_static::lazy_static;
use regex::Regex;
use serenity::all::Message;
use serenity::prelude::*;
use crate::utils;

lazy_static! {
    static ref CARD_REGEX: Regex = Regex::new(r"\[\[([a-zA-Z]+)]]").expect("Invalid regex");
}


pub async fn find_cards(msg: &Message, ctx: &Context) {
    for capture in CARD_REGEX.captures_iter(&msg.content) {
        let Some(name) = capture.get(1) else {
            continue;
        };
        utils::send(name.as_str(), &msg, &ctx).await
    }
}