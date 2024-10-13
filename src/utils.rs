pub(crate) mod fuzzy;

use log;
use serenity::all::{Context, CreateAttachment, CreateMessage, Message};

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

pub async fn send_image(image: &Vec<u8>, image_name: &String, msg: &Message, ctx: &Context) {
    let message = CreateMessage::new();
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
