use std::collections::HashMap;

use sqlx::Row;
use uuid::Uuid;

use crate::db::PSQL;
use crate::mtg::NewCardInfo;

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

impl PSQL {
    async fn add_to_legalities(&self, card: &NewCardInfo) -> Uuid {
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

    async fn add_to_rules(&self, card: &NewCardInfo, legalities_id: &Uuid) -> Uuid {
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

    async fn add_to_sets(&self, card: &NewCardInfo) {
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

    async fn add_to_cards(&self, card: &NewCardInfo, image_id: &Uuid, rules_id: &Uuid) {
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

    pub async fn add_card(&self, card: &NewCardInfo, image: &Vec<u8>) {
        let legalities_id = self.add_to_legalities(&card).await;
        let rules_id = self.add_to_rules(&card, &legalities_id).await;
        let image_id = self.add_to_images(&image).await;
        self.add_to_sets(&card).await;
        self.add_to_cards(&card, &image_id, &rules_id).await;

        log::info!("Added {} to postgres", card.name)
    }

    pub async fn fetch_card(&self, id: &str) -> Option<Vec<Vec<u8>>> {
        match sqlx::query(EXACT_MATCH)
            .bind(&id)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed card fetch - {why}");
                None
            }
            Ok(rows) => rows.into_iter().map(|row| row.get("png")).collect(),
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
