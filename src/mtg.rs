use serde::Deserialize;
use serenity::all::Message;
use serenity::futures::future::join_all;
use serenity::prelude::*;
use sqlx::Executor;
use bytes::Bytes;
use uuid::Uuid;
use base64::prelude::*;

use crate::mtg::response::CardResponse;
use crate::{utils, Handler};

mod response;

const SCRYFALL: &str = "https://api.scryfall.com/cards/named?fuzzy=";

impl Handler {
    pub async fn find_cards(&self, msg: &Message, ctx: &Context) {
        let futures: Vec<_> = self
            .card_regex
            .captures_iter(&msg.content)
            .filter_map(|capture| {
                if let Some(name) = capture.get(1) {
                    Some(self.find_card(name.as_str(), &msg, &ctx))
                } else {
                    None
                }
            })
            .collect();

        join_all(futures).await;
    }

    async fn find_card(&self, name: &str, msg: &Message, ctx: &Context) {
        println!("Searching scryfall for \"{}\"", name);
        let response = self
            .http_client
            .get(format!("{}{}", SCRYFALL, name.replace(" ", "+")))
            .send()
            .await
            .expect("Failed request");

        let card = if response.status().is_success() {
            match response.json::<CardResponse>().await {
                Ok(response) => response,
                Err(why) => {
                    println!("Error getting card from scryfall - {why:?}");
                    return;
                }
            }
        } else {
            return;
        };

        println!(
            "Matched with - \"{}\". Now searching for image...",
            card.name
        );

        let Ok(image) = self
            .http_client
            .get(&card.image_uris.png)
            .send()
            .await
            .expect("failed request")
            .bytes()
            .await
        else {
            return;
        };
        println!("Image found for - \"{}\".", &card.name);
        utils::send_image(&image, &format!("{}.png", &card.name), &msg, &ctx).await;
        self.add_to_postgres(card, image).await;
    }

    async fn add_to_postgres(
        &self,
        card: CardResponse,
        image: Bytes,
    ) {
        let image_id = Uuid::new_v4().to_string();
        let encoded = BASE64_STANDARD.encode(&image.to_vec());
        if let Err(why) = self
            .pg_pool
            .execute(sqlx::query(&format!(
                "INSERT INTO images values (uuid('{}'), decode('{}', 'base64'))",
                image_id, encoded
            )))
            .await
        {
            println!("Failed images insert - {why}")
        };
    }
}
