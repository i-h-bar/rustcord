use serde::Deserialize;
use serenity::all::Message;
use serenity::futures::future::join_all;
use serenity::prelude::*;
use sqlx::{Executor, Row};
use uuid::Uuid;

use crate::mtg::response::CardResponse;
use crate::{utils, Handler};

mod response;

const SCRYFALL: &str = "https://api.scryfall.com/cards/named?fuzzy=";
const IMAGE_INSERT: &str = r#"INSERT INTO images (id, png) values ($1, $2) ON CONFLICT DO NOTHING"#;
const SET_INSERT: &str =
    r#"INSERT INTO sets (id, name, code) values (uuid($1), $2, $3) ON CONFLICT DO NOTHING"#;
const CARD_INSERT: &str = r#"INSERT INTO cards (id, name, flavour_text, set_id, image_id, artist) values (uuid($1), $2, $3, uuid($4), uuid($5), $6) ON CONFLICT DO NOTHING"#;
const EXACT_MATCH: &str = r#"select png from cards join images on cards.image_id = images.id where cards.name = $1"#;

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
        if self.card_names.contains(name) {
            println!("Found exact match in cache!");
            let image = sqlx::query(EXACT_MATCH)
                .bind(&name)
                .fetch_one(&self.pg_pool)
                .await
                .unwrap()
                .get("png");
            utils::send_image(&image, &format!("{}.png", &name), &msg, &ctx).await;
        } else {
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
                .expect("failed image request")
                .bytes()
                .await
            else {
                println!("Failed to retrieve image bytes");
                return;
            };
            println!("Image found for - \"{}\".", &card.name);
            let image = image.to_vec();

            utils::send_image(&image, &format!("{}.png", &card.name), &msg, &ctx).await;
            self.add_to_postgres(card, image).await;
        }
    }

    async fn add_to_postgres(&self, card: CardResponse, image: Vec<u8>) {
        let image_id = Uuid::new_v4();
        let image_insert = sqlx::query(IMAGE_INSERT).bind(&image_id).bind(&image);

        let set_insert = sqlx::query(SET_INSERT)
            .bind(&card.set_id)
            .bind(&card.set_name)
            .bind(&card.set);

        let card_insert = sqlx::query(CARD_INSERT)
            .bind(&card.id)
            .bind(&card.name)
            .bind(&card.flavor_text)
            .bind(&card.set_id)
            .bind(&image_id)
            .bind(&card.artist);

        if let Err(why) = self.pg_pool.execute(image_insert).await {
            println!("Failed images insert - {why}")
        };
        if let Err(why) = self.pg_pool.execute(set_insert).await {
            println!("Failed set insert - {why}")
        };
        if let Err(why) = self.pg_pool.execute(card_insert).await {
            println!("Failed card insert - {why}")
        };
    }
}
