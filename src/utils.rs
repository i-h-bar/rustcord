use bytes::Bytes;
use serenity::all::{Context, CreateAttachment, CreateMessage, Message};

pub async fn send(content: &str, msg: &Message, ctx: &Context) {
    if let Err(why) = msg.channel_id.say(&ctx.http, content).await {
        println!("Error sending message - {why:?}")
    }
}

pub async fn send_image(
    image: &Vec<u8>,
    image_name: &String,
    msg: &Message,
    ctx: &Context,
) {
    let message = CreateMessage::new();
    let attachment = CreateAttachment::bytes(image.to_vec(), image_name);
    let message = message.add_file(attachment);
    if let Err(why) = msg.channel_id.send_message(&ctx.http, message).await {
        println!("Error sending image - {why:?}")
    }
}
