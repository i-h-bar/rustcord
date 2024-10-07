use serde::Deserialize;
use serenity::all::Message;
use serenity::prelude::*;

use crate::mtg::response::CardResponse;
use crate::{utils, Handler};

mod response;

const SCRYFALL: &str = "https://api.scryfall.com/cards/named?fuzzy=";

impl Handler {
    pub async fn find_cards(&self, msg: &Message, ctx: &Context) {
        for capture in self.card_regex.captures_iter(&msg.content) {
            let Some(name) = capture.get(1) else {
                continue;
            };
            println!("Searching scryfall for \"{}\"", name.as_str());
            let response = self
                .http_client
                .get(format!("{}{}", SCRYFALL, name.as_str().replace(" ", "+")))
                .send()
                .await
                .expect("Failed request");

            let card = if response.status().is_success() {
                match response.json::<CardResponse>().await {
                    Ok(response) => response,
                    Err(why) => {
                        println!("Error getting card from scryfall - {why:?}");
                        continue;
                    }
                }
            } else {
                continue;
            };

            println!(
                "Matched with - \"{}\". Now searching for image...",
                card.name
            );

            let Ok(image) = self
                .http_client
                .get(card.image_uris.png)
                .send()
                .await
                .expect("failed request")
                .bytes()
                .await
            else {
                continue;
            };
            println!("Image found for - \"{}\".", card.name);
            utils::send_image(image, format!("{}.png", card.name), &msg, &ctx).await;
        }
    }
}
