use std::collections::HashSet;

use serde::Deserialize;
use serenity::all::Message;
use serenity::futures::future::join_all;
use serenity::prelude::*;
use rayon::prelude::*;
use sqlx::{Executor, Row};
use tokio::time::Instant;
use uuid::Uuid;

use crate::{Handler, utils};
use crate::mtg::response::CardResponse;

mod response;

const SCRYFALL: &str = "https://api.scryfall.com/cards/named?fuzzy=";
const IMAGE_INSERT: &str = r#"INSERT INTO images (id, png) values ($1, $2) ON CONFLICT DO NOTHING"#;
const SET_INSERT: &str =
    r#"INSERT INTO sets (id, name, code) values (uuid($1), $2, $3) ON CONFLICT DO NOTHING"#;
const CARD_INSERT: &str = r#"INSERT INTO cards (id, name, flavour_text, set_id, image_id, artist) values (uuid($1), $2, $3, uuid($4), uuid($5), $6) ON CONFLICT DO NOTHING"#;

const EXACT_MATCH: &str =
    r#"select png from cards join images on cards.image_id = images.id where cards.name = $1"#;

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
        let start = Instant::now();

        let normalised_name = name.to_lowercase();
        let cards_vec = sqlx::query("select name from cards")
            .fetch_all(&self.pg_pool)
            .await
            .expect("Failed to get cards names");

        let card_names = cards_vec.into_par_iter().map(|row| row.get("name")).collect::<HashSet<String>>();

        if card_names.contains(&normalised_name) {
            println!("Found exact match in cache!");
            let image = sqlx::query(EXACT_MATCH)
                .bind(&normalised_name)
                .fetch_one(&self.pg_pool)
                .await
                .expect("Couldn't find card in db even though it was there before")
                .get("png");

            println!("Found '{}' locally in {:.2?}", normalised_name, start.elapsed());
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

            println!("Found from '{}' from scryfall in {:.2?}", card.name, start.elapsed());
            utils::send_image(&image, &format!("{}.png", &card.name), &msg, &ctx).await;
            self.add_to_postgres(card, image).await;
        }
    }

    async fn add_to_postgres(&self, card: CardResponse, image: Vec<u8>) {
        let image_id = Uuid::new_v4();
        if let Err(why) = sqlx::query(IMAGE_INSERT)
            .bind(&image_id)
            .bind(&image)
            .execute(&self.pg_pool)
            .await
        {
            println!("Failed images insert - {why}")
        };

        if let Err(why) = sqlx::query(SET_INSERT)
            .bind(&card.set_id)
            .bind(&card.set_name)
            .bind(&card.set)
            .execute(&self.pg_pool)
            .await
        {
            println!("Failed set insert - {why}")
        };

        if let Err(why) = sqlx::query(CARD_INSERT)
            .bind(&card.id)
            .bind(&card.name.to_lowercase())
            .bind(&card.flavor_text)
            .bind(&card.set_id)
            .bind(&image_id)
            .bind(&card.artist)
            .execute(&self.pg_pool)
            .await
        {
            println!("Failed card insert - {why}")
        };
    }
}
