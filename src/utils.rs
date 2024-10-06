use serenity::all::{Context, Message};

pub async fn send(content: &str, msg: &Message, ctx: &Context) {
    if let Err(why) = msg.channel_id.say(&ctx.http, content).await {
        println!("Error sending message - {why:?}")
    }
}
