use sqlx::postgres::PgRow;
use sqlx::{Error, FromRow, Row};
use std::collections::HashMap;
use uuid::Uuid;

use crate::db::PSQL;
use crate::mtg::{CardInfo, FoundCard};

const LEGALITIES_INSERT: &str = r#"
INSERT INTO legalities
(id, alchemy, brawl, commander, duel, explorer, future, gladiator, historic, legacy, modern,
oathbreaker, oldschool, pauper, paupercommander, penny, pioneer, predh, premodern, standard,
standardbrawl, timeless, vintage)
values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
$15, $16, $17, $18, $19, $20, $21, $22, $23) ON CONFLICT DO NOTHING"#;
const RULES_INSERT: &str = r#"
INSERT INTO rules
(id, colour_identity, cmc, power, toughness, type_line, oracle_text, keywords, loyalty, defence, mana_cost, legalities_id)
values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) ON CONFLICT DO NOTHING
"#;
const IMAGE_INSERT: &str = r#"INSERT INTO images (id, png) values ($1, $2) ON CONFLICT DO NOTHING"#;
const SET_INSERT: &str =
    r#"INSERT INTO sets (id, name, code) values (uuid($1), $2, $3) ON CONFLICT DO NOTHING"#;
const CARD_INSERT: &str = r#"
INSERT INTO cards (id, name, flavour_text, set_id, image_id, artist, rules_id, other_side)
values (uuid($1), $2, $3, uuid($4), uuid($5), $6, $7, uuid($8)) ON CONFLICT DO NOTHING"#;

const EXACT_MATCH: &str = r#"
select png from cards join images on cards.image_id = images.id where cards.id = uuid($1) or cards.other_side = uuid($1)
"#;

const FUZZY_FIND: &str = r#"
select name,
       png,
       other_side,
       similarity(cards.name, $1) as sml
from cards join images on cards.image_id = images.id
order by sml desc
limit 1;
"#;

const BACKSIDE_FIND: &str = r#"
select png, name, other_side from cards join images on cards.image_id = images.id where cards.id = uuid($1)
"#;

pub struct FuzzyFound {
    pub(crate) png: Vec<u8>,
    pub(crate) name: String,
    pub(crate) similarity: f32,
    pub(crate) other_side: Option<String>,
}

impl<'r> FromRow<'r, PgRow> for FuzzyFound {
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        let other_side = match row.try_get::<Option<Uuid>, &str>("other_side") {
            Ok(id) => match id {
                Some(uuid) => Some(uuid.to_string()),
                None => None,
            },
            Err(why) => {
                log::warn!("Failed to get other side - {why}");
                None
            }
        };

        Ok(FuzzyFound {
            png: row.get::<Vec<u8>, &str>("png"),
            name: row.get::<String, &str>("name"),
            similarity: row.try_get::<f32, &str>("sml").unwrap_or_else(|_| 0.0),
            other_side,
        })
    }
}

impl PSQL {
    async fn add_to_legalities(&self, card: &CardInfo) -> Uuid {
        let legalities_id = Uuid::new_v4();
        if let Err(why) = sqlx::query(LEGALITIES_INSERT)
            .bind(&legalities_id)
            .bind(&card.legalities.alchemy)
            .bind(&card.legalities.brawl)
            .bind(&card.legalities.commander)
            .bind(&card.legalities.duel)
            .bind(&card.legalities.explorer)
            .bind(&card.legalities.future)
            .bind(&card.legalities.gladiator)
            .bind(&card.legalities.historic)
            .bind(&card.legalities.legacy)
            .bind(&card.legalities.modern)
            .bind(&card.legalities.oathbreaker)
            .bind(&card.legalities.oldschool)
            .bind(&card.legalities.pauper)
            .bind(&card.legalities.paupercommander)
            .bind(&card.legalities.penny)
            .bind(&card.legalities.pioneer)
            .bind(&card.legalities.predh)
            .bind(&card.legalities.premodern)
            .bind(&card.legalities.standard)
            .bind(&card.legalities.standardbrawl)
            .bind(&card.legalities.timeless)
            .bind(&card.legalities.vintage)
            .execute(&self.pool)
            .await
        {
            log::warn!("Failed legalities insert - {why}")
        }

        legalities_id
    }

    async fn add_to_rules(&self, card: &CardInfo, legalities_id: &Uuid) -> Uuid {
        let rules_id = Uuid::new_v4();
        if let Err(why) = sqlx::query(RULES_INSERT)
            .bind(&rules_id)
            .bind(&card.colour_identity)
            .bind(&card.cmc)
            .bind(&card.power)
            .bind(&card.toughness)
            .bind(&card.type_line)
            .bind(&card.oracle_text)
            .bind(&card.keywords)
            .bind(&card.loyalty)
            .bind(&card.defence)
            .bind(&card.mana_cost)
            .bind(&legalities_id)
            .execute(&self.pool)
            .await
        {
            log::warn!("Failed legalities insert - {why}")
        }

        rules_id
    }

    async fn add_to_images(&self, image: &Vec<u8>) -> Uuid {
        let image_id = Uuid::new_v4();
        if let Err(why) = sqlx::query(IMAGE_INSERT)
            .bind(&image_id)
            .bind(&image)
            .execute(&self.pool)
            .await
        {
            log::warn!("Failed images insert - {why}")
        };

        image_id
    }

    async fn add_to_sets(&self, card: &CardInfo) {
        if let Err(why) = sqlx::query(SET_INSERT)
            .bind(&card.set_id)
            .bind(&card.set_name)
            .bind(&card.set_code)
            .execute(&self.pool)
            .await
        {
            log::warn!("Failed set insert - {why}")
        };
    }

    async fn add_to_cards(&self, card: &CardInfo, image_id: &Uuid, rules_id: &Uuid) {
        if let Err(why) = sqlx::query(CARD_INSERT)
            .bind(&card.card_id)
            .bind(&card.name)
            .bind(&card.flavour_text)
            .bind(&card.set_id)
            .bind(&image_id)
            .bind(&card.artist)
            .bind(&rules_id)
            .bind(&card.other_side)
            .execute(&self.pool)
            .await
        {
            log::warn!("Failed card insert - {why}")
        };
    }

    pub async fn add_card<'a>(&'a self, card: &FoundCard<'a>) {
        if let Some(card_info) = &card.front {
            let legalities_id = self.add_to_legalities(&card_info).await;
            let rules_id = self.add_to_rules(&card_info, &legalities_id).await;
            let image_id = self.add_to_images(&card.image).await;
            self.add_to_sets(&card_info).await;
            self.add_to_cards(&card_info, &image_id, &rules_id).await;

            log::info!("Added {} to postgres", card_info.name)
        }

        if let Some(card_info) = &card.back {
            let image_id = if let Some(back_image) = &card.back_image {
                self.add_to_images(&back_image).await
            } else {
                log::warn!("Could not add the back image as there was none to add");
                return;
            };

            let legalities_id = self.add_to_legalities(&card_info).await;
            let rules_id = self.add_to_rules(&card_info, &legalities_id).await;
            self.add_to_sets(&card_info).await;
            self.add_to_cards(&card_info, &image_id, &rules_id).await;

            log::info!("Added {} to postgres", card_info.name)
        }
    }

    pub async fn fetch_backside(&self, card_id: &str) -> Option<FuzzyFound> {
        match sqlx::query(BACKSIDE_FIND)
            .bind(&card_id)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed card fetch - {why}");
                None
            }
            Ok(row) => FuzzyFound::from_row(&row).ok(),
        }
    }

    pub async fn fuzzy_fetch(&self, name: &str) -> Option<FuzzyFound> {
        match sqlx::query(FUZZY_FIND)
            .bind(&name)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed card fetch - {why}");
                None
            }
            Ok(row) => FuzzyFound::from_row(&row).ok(),
        }
    }

    pub async fn names_and_ids(&self) -> HashMap<String, String> {
        match sqlx::query("select cards.name, cards.id from cards")
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(|row| (row.get("name"), row.get::<Uuid, &str>("id").to_string()))
                .collect::<HashMap<String, String>>(),
            Err(why) => {
                log::warn!("Failed card fetch - {why}");
                HashMap::new()
            }
        }
    }
}
