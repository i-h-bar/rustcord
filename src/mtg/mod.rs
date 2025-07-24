use crate::app::search::CardAndImage;
use crate::clients::MessageInteraction;
use crate::image_store::Images;
use crate::mtg::card::FuzzyFound;
use crate::utils;
use serenity::all::{Context, CreateAttachment, CreateMessage, Message};

pub mod card;

pub async fn card_response<MI: MessageInteraction>(card: Option<CardAndImage>, interaction: &MI) {
    match card {
        None => {
            if let Err(why) = interaction
                .reply(String::from("Failed to find card :("))
                .await
            {
                log::error!("Error sending card not found message :( {:?}", why);
            }
        }
        Some((card, images)) => {
            if let Err(why) = interaction.send_card(card, images).await {
                log::error!("Error sending card message :( {:?}", why);
            };
        }
    }
}
