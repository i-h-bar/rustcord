use crate::ports::storage::{
    Artist, Card, CardInfo, Illustration, Image, Legality, Price, Rule, Set, Storage,
};
use async_trait::async_trait;
use futures::future;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Row};
use std::env;

pub struct Postgres {
    pool: Pool<sqlx::Postgres>,
}

impl Postgres {
    /// # Panics
    ///
    /// Panics if `POSTGRES_USER`, `POSTGRES_PW`, or `POSTGRES_DB` env vars are not set,
    /// or if the connection to Postgres cannot be established.
    pub async fn create() -> Self {
        let user = env::var("POSTGRES_USER").expect("POSTGRES_USER wasn't in env vars");
        let password = env::var("POSTGRES_PW").expect("POSTGRES_PW wasn't in env vars");
        let db = env::var("POSTGRES_DB").expect("POSTGRES_DB wasn't in env vars");
        let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost:5432".to_string());
        let uri = format!("postgresql://{user}:{password}@{host}/{db}");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&uri)
            .await
            .expect("Failed Postgres connection");

        Self { pool }
    }

    async fn get_set_volume(&self, set: &Set) -> u32 {
        match sqlx::query("select count(*) from set where id = $1")
            .bind(set.id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(result) => {
                u32::try_from(result.try_get::<i64, &str>("count").unwrap_or(0)).unwrap_or(0)
            }
            Err(why) => {
                log::warn!("Failed to fetch volume: {why}");
                0
            }
        }
    }

    async fn upsert_artist(&self, artist: &Artist) {
        if let Err(e) = sqlx::query(
            "INSERT INTO artist (id, name, normalised_name)
             VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        )
        .bind(artist.id)
        .bind(&artist.name)
        .bind(&artist.normalised_name)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert artist {}: {}", artist.id, e);
        }
    }

    async fn upsert_image(&self, image: &Image) {
        if let Err(e) = sqlx::query(
            "INSERT INTO image (id, scryfall_url) VALUES ($1, $2)
             ON CONFLICT (id) DO UPDATE SET scryfall_url = EXCLUDED.scryfall_url
             WHERE image.scryfall_url IS DISTINCT FROM EXCLUDED.scryfall_url",
        )
        .bind(image.id)
        .bind(&image.scryfall_url)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert image {}: {}", image.id, e);
        }
    }

    async fn upsert_illustration(&self, illustration: &Illustration) {
        if let Err(e) = sqlx::query(
            "INSERT INTO illustration (id, scryfall_url)
             VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(illustration.id)
        .bind(&illustration.scryfall_url)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert illustration {}: {}", illustration.id, e);
        }
    }

    async fn upsert_set(&self, set: &Set) {
        if let Err(e) = sqlx::query(
            "INSERT INTO set (id, name, normalised_name, abbreviation)
             VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
        )
        .bind(set.id)
        .bind(&set.name)
        .bind(&set.normalised_name)
        .bind(&set.abbreviation)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert set {}: {}", set.id, e);
        }
    }

    async fn upsert_rule(&self, rule: &Rule) {
        if let Err(e) = sqlx::query(
            "INSERT INTO rule
             (id, colour_identity, mana_cost, cmc, power, toughness, loyalty, defence,
              type_line, oracle_text, colours, keywords, produced_mana, rulings_url)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
             ON CONFLICT(id) DO UPDATE SET
               colour_identity = EXCLUDED.colour_identity,
               mana_cost       = EXCLUDED.mana_cost,
               cmc             = EXCLUDED.cmc,
               power           = EXCLUDED.power,
               toughness       = EXCLUDED.toughness,
               loyalty         = EXCLUDED.loyalty,
               defence         = EXCLUDED.defence,
               type_line       = EXCLUDED.type_line,
               oracle_text     = EXCLUDED.oracle_text,
               colours         = EXCLUDED.colours,
               keywords        = EXCLUDED.keywords,
               produced_mana   = EXCLUDED.produced_mana,
               rulings_url     = EXCLUDED.rulings_url",
        )
        .bind(rule.id)
        .bind(&rule.colour_identity)
        .bind(&rule.mana_cost)
        .bind(rule.cmc)
        .bind(&rule.power)
        .bind(&rule.toughness)
        .bind(&rule.loyalty)
        .bind(&rule.defence)
        .bind(&rule.type_line)
        .bind(&rule.oracle_text)
        .bind(&rule.colours)
        .bind(&rule.keywords)
        .bind(&rule.produced_mana)
        .bind(&rule.rulings_url)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert rule {}: {}", rule.id, e);
        }
    }

    async fn upsert_legality(&self, leg: &Legality) {
        if let Err(e) = sqlx::query(
            "INSERT INTO legality
             (id, alchemy, brawl, commander, duel, future, gladiator, historic, legacy, modern,
              oathbreaker, oldschool, pauper, paupercommander, penny, pioneer, predh, premodern,
              standard, standardbrawl, timeless, vintage, game_changer)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                     $17, $18, $19, $20, $21, $22, $23)
             ON CONFLICT(id) DO UPDATE SET
               alchemy         = EXCLUDED.alchemy,
               brawl           = EXCLUDED.brawl,
               commander       = EXCLUDED.commander,
               duel            = EXCLUDED.duel,
               future          = EXCLUDED.future,
               gladiator       = EXCLUDED.gladiator,
               historic        = EXCLUDED.historic,
               legacy          = EXCLUDED.legacy,
               modern          = EXCLUDED.modern,
               oathbreaker     = EXCLUDED.oathbreaker,
               oldschool       = EXCLUDED.oldschool,
               pauper          = EXCLUDED.pauper,
               paupercommander = EXCLUDED.paupercommander,
               penny           = EXCLUDED.penny,
               pioneer         = EXCLUDED.pioneer,
               predh           = EXCLUDED.predh,
               premodern       = EXCLUDED.premodern,
               standard        = EXCLUDED.standard,
               standardbrawl   = EXCLUDED.standardbrawl,
               timeless        = EXCLUDED.timeless,
               vintage         = EXCLUDED.vintage,
               game_changer    = EXCLUDED.game_changer",
        )
        .bind(leg.id)
        .bind(&leg.alchemy)
        .bind(&leg.brawl)
        .bind(&leg.commander)
        .bind(&leg.duel)
        .bind(&leg.future)
        .bind(&leg.gladiator)
        .bind(&leg.historic)
        .bind(&leg.legacy)
        .bind(&leg.modern)
        .bind(&leg.oathbreaker)
        .bind(&leg.oldschool)
        .bind(&leg.pauper)
        .bind(&leg.paupercommander)
        .bind(&leg.penny)
        .bind(&leg.pioneer)
        .bind(&leg.predh)
        .bind(&leg.premodern)
        .bind(&leg.standard)
        .bind(&leg.standardbrawl)
        .bind(&leg.timeless)
        .bind(&leg.vintage)
        .bind(leg.game_changer)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert legality {}: {}", leg.id, e);
        }
    }

    async fn upsert_card(&self, card: &Card) {
        if let Err(e) = sqlx::query(
            "INSERT INTO card
             (id, oracle_id, name, normalised_name, scryfall_url, flavour_text, release_date,
              reserved, rarity, artist_id, image_id, illustration_id, set_id, backside_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
             ON CONFLICT (id) DO UPDATE SET
               normalised_name = EXCLUDED.normalised_name,
               scryfall_url    = EXCLUDED.scryfall_url,
               reserved        = EXCLUDED.reserved,
               oracle_id       = EXCLUDED.oracle_id
             WHERE (card.normalised_name IS DISTINCT FROM EXCLUDED.normalised_name OR
                    card.scryfall_url    IS DISTINCT FROM EXCLUDED.scryfall_url    OR
                    card.reserved        IS DISTINCT FROM EXCLUDED.reserved        OR
                    card.oracle_id       IS DISTINCT FROM EXCLUDED.oracle_id)",
        )
        .bind(card.id)
        .bind(card.oracle_id)
        .bind(&card.name)
        .bind(&card.normalised_name)
        .bind(&card.scryfall_url)
        .bind(&card.flavour_text)
        .bind(card.release_date)
        .bind(card.reserved)
        .bind(&card.rarity)
        .bind(card.artist_id)
        .bind(card.image_id)
        .bind(card.illustration_id)
        .bind(card.set_id)
        .bind(card.backside_id)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert card {}: {}", card.id, e);
        }
    }

    async fn upsert_price(&self, price: &Price) {
        if let Err(e) = sqlx::query(
            "INSERT INTO price (id, usd, usd_foil, usd_etched, euro, euro_foil, tix, updated_time)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (id) DO UPDATE SET
               usd          = EXCLUDED.usd,
               usd_foil     = EXCLUDED.usd_foil,
               usd_etched   = EXCLUDED.usd_etched,
               euro         = EXCLUDED.euro,
               euro_foil    = EXCLUDED.euro_foil,
               tix          = EXCLUDED.tix,
               updated_time = EXCLUDED.updated_time",
        )
        .bind(price.id)
        .bind(price.usd)
        .bind(price.usd_foil)
        .bind(price.usd_etched)
        .bind(price.euro)
        .bind(price.euro_foil)
        .bind(price.tix)
        .bind(price.updated_time)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert price {}: {}", price.id, e);
        }
    }

    async fn upsert_card_info(&self, info: CardInfo) {
        self.upsert_artist(&info.artist).await;
        self.upsert_image(&info.image).await;
        if let Some(ill) = &info.illustration {
            self.upsert_illustration(ill).await;
        }
        self.upsert_set(&info.set).await;
        self.upsert_rule(&info.rule).await;
        self.upsert_legality(&info.legality).await;
        self.upsert_card(&info.card).await;
        self.upsert_price(&info.price).await;
    }
}

#[async_trait]
impl Storage for Postgres {
    async fn get_set_volumes(&self, sets: Vec<Set>) -> Vec<(Set, u32)> {
        future::join_all(sets.into_iter().map(|set| async {
            let volume = self.get_set_volume(&set).await;
            (set, volume)
        }))
        .await
    }

    async fn upsert_cards(&self, cards: Vec<CardInfo>) {
        future::join_all(
            cards
                .into_iter()
                .map(|card_info| self.upsert_card_info(card_info)),
        )
        .await;
    }
}
