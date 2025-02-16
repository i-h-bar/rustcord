use crate::db::PSQL;
use crate::mtg::{CardInfo, FoundCard};
use crate::utils;
use regex::Captures;
use sqlx::postgres::PgRow;
use sqlx::{Error, FromRow, Row};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

const RULES_LEGAL_IDS: &str = r#"
select
    distinct rules.id as rules_id,
             legalities.id as lagalities_id
from cards
    join rules on rules.id = cards.rules_id
    join legalities on legalities.id = rules.legalities_id
where cards.name = $1
"#;

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

const FUZZY_FIND: &str = r#"
select cards.name,
       png,
       other_side,
       similarity(cards.name, $1) as sml
from cards join images on cards.image_id = images.id
join sets on cards.set_id = sets.id
where ($4 is null or similarity(cards.artist, $4) > 0.5)
and ($3 is null or similarity(sets.name, $3) > 0.75)
and ($2 is null or sets.code = $2)
order by sml desc
limit 1;
"#;

const BACKSIDE_FIND: &str = r#"
select png, name, other_side from cards join images on cards.image_id = images.id where cards.id = uuid($1)
"#;

pub struct FuzzyFound {
    pub(crate) png: Vec<u8>,
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
            similarity: row.try_get::<f32, &str>("sml").unwrap_or_else(|_| 0.0),
            other_side,
        })
    }
}

impl PSQL {
    async fn add_to_legalities(&self, card: &CardInfo, legalities_id: &Uuid) {
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
    }

    async fn add_to_rules(&self, card: &CardInfo, rules_id: &Uuid, legalities_id: &Uuid) {
        let keywords: Option<Vec<&str>> = match &card.keywords.deref() {
            Some(keywords) => Some(
                keywords.iter().map(| keyword | keyword.as_ref() ).collect()
            ),
            None => None
        };

        if let Err(why) = sqlx::query(RULES_INSERT)
            .bind(&rules_id)
            .bind(card.colour_identity.deref())
            .bind(&card.cmc)
            .bind(card.power.deref())
            .bind(card.toughness.deref())
            .bind(card.type_line.deref())
            .bind(card.oracle_text.deref())
            .bind(keywords)
            .bind(card.loyalty.deref())
            .bind(card.defence.deref())
            .bind(card.mana_cost.deref())
            .bind(&legalities_id)
            .execute(&self.pool)
            .await
        {
            log::warn!("Failed legalities insert - {why}")
        }
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
            .bind(&card.set_id.deref())
            .bind(utils::normalise(&card.set_name.deref()))
            .bind(utils::normalise(&card.set_code.deref()))
            .execute(&self.pool)
            .await
        {
            log::warn!("Failed set insert - {why}")
        };
    }

    async fn add_to_cards(&self, card: &CardInfo, image_id: &Uuid, rules_id: &Uuid) {
        let other_side = match &card.other_side {
            Some(other_side) => Some(other_side.deref()),
            None => None
        };

        if let Err(why) = sqlx::query(CARD_INSERT)
            .bind(card.card_id.deref())
            .bind(utils::normalise(card.name.deref()))
            .bind(card.flavour_text.deref())
            .bind(&card.set_id.deref())
            .bind(&image_id)
            .bind(utils::normalise(&card.artist.deref()))
            .bind(&rules_id)
            .bind(other_side)
            .execute(&self.pool)
            .await
        {
            log::warn!("Failed card insert - {why}")
        };
    }

    pub async fn add_card<'a>(
        &'a self,
        card: &FoundCard<'a>,
        shared_ids: &HashMap<&str, (Uuid, Uuid)>,
    ) {
        if let Some(card_info) = &card.front {
            let Some((legalities_id, rules_id)) = shared_ids.get(&card_info.name.as_ref()) else {
                log::warn!("No front ids found");
                return;
            };

            self.add_to_legalities(&card_info, &legalities_id).await;
            self.add_to_rules(&card_info, &rules_id, &legalities_id)
                .await;
            let image_id = self.add_to_images(&card.image).await;
            self.add_to_sets(&card_info).await;
            self.add_to_cards(&card_info, &image_id, &rules_id).await;
            log::info!("Added {} to postgres", card_info.name);
        } else {
            log::warn!("No front card found");
            return;
        };

        if let Some(card_info) = &card.back {
            let Some((legalities_id, rules_id)) = shared_ids.get(&card_info.name.as_ref()) else {
                log::warn!("No front back ids found");
                return;
            };

            let image_id = if let Some(back_image) = &card.back_image {
                self.add_to_images(&back_image).await
            } else {
                log::warn!("Could not add the back image as there was none to add");
                return;
            };

            self.add_to_rules(&card_info, &rules_id, &legalities_id)
                .await;
            self.add_to_sets(&card_info).await;
            self.add_to_cards(&card_info, &image_id, &rules_id).await;

            log::info!("Added {} to postgres", card_info.name)
        }
    }

    pub async fn fetch_rules_legalities_id(&self, name: &str) -> Option<(Uuid, Uuid)> {
        if let Ok(result) = sqlx::query(RULES_LEGAL_IDS)
            .bind(&name)
            .fetch_one(&self.pool)
            .await
        {
            Some((
                result.get::<Uuid, &str>("rules_id"),
                result.get::<Uuid, &str>("legalities_id"),
            ))
        } else {
            None
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

    pub async fn fuzzy_fetch(&self, query: Arc<QueryParams<'_>>) -> Option<FuzzyFound> {
        match sqlx::query(FUZZY_FIND)
            .bind(&query.name)
            .bind(&query.set_code)
            .bind(&query.set_name)
            .bind(&query.artist)
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
}

pub struct QueryParams<'a> {
    pub name: String,
    pub raw_name: &'a str,
    pub set_code: Option<String>,
    pub set_name: Option<String>,
    pub artist: Option<String>,
}

impl<'a> QueryParams<'a> {
    pub fn from(capture: Captures<'a>) -> Option<Self> {
        let raw_name = capture.get(1)?.as_str();
        let name = utils::normalise(&raw_name);
        let (set_code, set_name) = match capture.get(4) {
            Some(set) => {
                let set = set.as_str();
                if set.chars().count() < 5 {
                    (Some(utils::normalise(set)), None)
                } else {
                    (None, Some(utils::normalise(set)))
                }
            }
            None => (None, None),
        };

        let artist = match capture.get(7) {
            Some(artist) => Some(utils::normalise(&artist.as_str())),
            None => None,
        };

        Some(Self {
            name,
            raw_name,
            artist,
            set_code,
            set_name,
        })
    }
}
