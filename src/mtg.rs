use serde::Deserialize;
use serenity::all::Message;
use serenity::futures::future::join_all;
use serenity::prelude::*;
use sqlx::Executor;
use uuid::Uuid;

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
        let image = image.to_vec();

        utils::send_image(&image, &format!("{}.png", &card.name), &msg, &ctx).await;
        self.add_to_postgres(card, image).await;
    }

    async fn add_to_postgres(
        &self,
        card: CardResponse,
        image: Vec<u8>,
    ) {
        let image_id = Uuid::new_v4();
        let image_insert = sqlx::query(r#"INSERT INTO images (id, png) values ($1, $2)"#)
            .bind(&image_id)
            .bind(&image);

        let set_insert = sqlx::query(r#"INSERT INTO sets (id, name, code) values (uuid($1), $2, $3) ON CONFLICT DO NOTHING"#)
            .bind(&card.set_id)
            .bind(&card.set_name)
            .bind(&card.set);

        let card_insert = sqlx::query(r#"INSERT INTO cards (id, name, flavour_text, set_id, image_id, artist) values (uuid($1), $2, $3, uuid($4), uuid($5), $6)"#)
            .bind(&card.id)
            .bind(&card.name)
            .bind(&card.flavor_text)
            .bind(&card.set_id)
            .bind(&image_id)
            .bind(&card.artist);

        if let Err(why) = self
            .pg_pool
            .execute(image_insert)
            .await
        {
            println!("Failed images insert - {why}")
        };
        if let Err(why) = self
            .pg_pool
            .execute(set_insert)
            .await
        {
            println!("Failed set insert - {why}")
        };
        if let Err(why) = self
            .pg_pool
            .execute(card_insert)
            .await
        {
            println!("Failed card insert - {why}")
        };
    }
}
