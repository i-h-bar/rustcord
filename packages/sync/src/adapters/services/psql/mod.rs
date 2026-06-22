#[cfg(feature = "local-dev")]
use indicatif::{ProgressBar, ProgressStyle};

use crate::ports::storage::{
    Artist, Card, CardInfo, Combo, Illustration, Image, Legality, Price, RelatedToken, Rule, Set,
    Storage, UpsertResult,
};
use async_trait::async_trait;
use futures::StreamExt;
use futures::future::{self, Either};
use sqlx::{Pool, Row, error::DatabaseError, postgres::PgPoolOptions};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use uuid::Uuid;

const BOT_RESERVE: usize = 5;
const FALLBACK_POOL_SIZE: usize = 5;

pub struct Postgres {
    pool: Pool<sqlx::Postgres>,
    pool_size: usize,
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

        let pool_size = Self::compute_pool_size(&uri).await;
        log::info!("Using Postgres pool size: {pool_size}");

        let pool = PgPoolOptions::new()
            .max_connections(pool_size as u32)
            .connect(&uri)
            .await
            .expect("Failed Postgres connection");

        Self { pool, pool_size }
    }

    async fn compute_pool_size(uri: &str) -> usize {
        let probe = match PgPoolOptions::new().max_connections(1).connect(uri).await {
            Ok(p) => p,
            Err(_) => return FALLBACK_POOL_SIZE,
        };

        let max: String = sqlx::query_scalar("SHOW max_connections")
            .fetch_one(&probe)
            .await
            .unwrap_or_else(|_| FALLBACK_POOL_SIZE.to_string());
        let max: usize = max.trim().parse().unwrap_or(FALLBACK_POOL_SIZE);

        let in_use: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_stat_activity WHERE pid != pg_backend_pid()",
        )
        .fetch_one(&probe)
        .await
        .unwrap_or(0);

        probe.close().await;

        let pool_size = max
            .saturating_sub(in_use as usize)
            .saturating_sub(BOT_RESERVE)
            .max(1);

        pool_size
    }

    async fn get_card_ids_for_set(&self, set: &Set) -> HashSet<Uuid> {
        match sqlx::query("SELECT id FROM card WHERE set_id = $1")
            .bind(set.id)
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => rows
                .iter()
                .filter_map(|r| r.try_get::<Uuid, &str>("id").ok())
                .collect(),
            Err(why) => {
                log::warn!("Failed to fetch card ids for set {}: {why}", set.id);
                HashSet::new()
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
             ON CONFLICT (id) DO UPDATE SET scryfall_url = EXCLUDED.scryfall_url",
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
            "INSERT INTO illustration (id, scryfall_url) VALUES ($1, $2)
             ON CONFLICT (id) DO UPDATE SET scryfall_url = EXCLUDED.scryfall_url",
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

    async fn upsert_card(&self, card: &Card) -> (Option<Uuid>, Option<Uuid>) {
        match sqlx::query_as::<_, (Option<Uuid>, Option<Uuid>)>(
            "WITH prev AS (
                SELECT image_id, illustration_id FROM card WHERE id = $1
            )
            INSERT INTO card
             (id, oracle_id, name, normalised_name, scryfall_url, flavour_text, release_date,
              reserved, rarity, artist_id, image_id, illustration_id, set_id, backside_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
             ON CONFLICT (id) DO UPDATE SET
               normalised_name = EXCLUDED.normalised_name,
               scryfall_url    = EXCLUDED.scryfall_url,
               reserved        = EXCLUDED.reserved,
               oracle_id       = EXCLUDED.oracle_id,
               image_id        = EXCLUDED.image_id,
               illustration_id = EXCLUDED.illustration_id
             WHERE (card.normalised_name IS DISTINCT FROM EXCLUDED.normalised_name  OR
                    card.scryfall_url     IS DISTINCT FROM EXCLUDED.scryfall_url     OR
                    card.reserved         IS DISTINCT FROM EXCLUDED.reserved         OR
                    card.oracle_id        IS DISTINCT FROM EXCLUDED.oracle_id        OR
                    card.image_id         IS DISTINCT FROM EXCLUDED.image_id         OR
                    card.illustration_id  IS DISTINCT FROM EXCLUDED.illustration_id)
             RETURNING
               (SELECT image_id FROM prev) AS prev_image_id,
               (SELECT illustration_id FROM prev) AS prev_illustration_id",
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
        .fetch_optional(&self.pool)
        .await
        {
            Ok(Some((prev_img, prev_ill))) => {
                let orphaned_img = prev_img.filter(|&old| old != card.image_id);
                let orphaned_ill = prev_ill.filter(|old| Some(*old) != card.illustration_id);
                (orphaned_img, orphaned_ill)
            }
            Ok(None) => (None, None),
            Err(e) => {
                log::warn!("Failed to upsert card {}: {}", card.id, e);
                (None, None)
            }
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

    async fn upsert_combo(&self, combo: &Combo) {
        if let Err(e) = sqlx::query(
            "INSERT INTO combo (id, card_id, combo_card_id) VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
        )
        .bind(combo.id)
        .bind(combo.card_id)
        .bind(combo.combo_card_id)
        .execute(&self.pool)
        .await
            && e.as_database_error()
                .and_then(DatabaseError::code)
                .as_deref()
                != Some("23503")
        {
            log::warn!("Failed to upsert combo {}: {}", combo.id, e);
        }
    }

    async fn upsert_related_token(&self, token: &RelatedToken) {
        if let Err(e) = sqlx::query(
            "INSERT INTO related_token (id, card_id, token_id) VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
        )
        .bind(token.id)
        .bind(token.card_id)
        .bind(token.token_id)
        .execute(&self.pool)
        .await
            && e.as_database_error()
                .and_then(DatabaseError::code)
                .as_deref()
                != Some("23503")
        {
            log::warn!("Failed to upsert related_token {}: {}", token.id, e);
        }
    }

    async fn upsert_card_info(&self, info: &CardInfo) -> (Option<Uuid>, Option<Uuid>) {
        self.upsert_artist(&info.artist).await;
        self.upsert_image(&info.image).await;
        if let Some(ill) = &info.illustration {
            self.upsert_illustration(ill).await;
        }
        self.upsert_set(&info.set).await;
        self.upsert_rule(&info.rule).await;
        self.upsert_legality(&info.legality).await;
        let (orphaned_img, orphaned_ill) = self.upsert_card(&info.card).await;
        self.upsert_price(&info.price).await;

        (orphaned_img, orphaned_ill)
    }
}

#[async_trait]
impl Storage for Postgres {
    async fn get_existing_card_ids(&self, sets: Vec<Set>) -> Vec<(Set, HashSet<Uuid>)> {
        future::join_all(sets.into_iter().map(|set| async {
            let ids = self.get_card_ids_for_set(&set).await;
            (set, ids)
        }))
        .await
    }

    async fn upsert_cards(&self, cards: &[CardInfo]) -> UpsertResult {
        log::info!("Upserting {} cards", cards.len());

        #[cfg(feature = "local-dev")]
        let pb = {
            let bar = ProgressBar::new(cards.len() as u64);
            bar.set_style(
                ProgressStyle::with_template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} cards ({eta})",
                )
                .unwrap()
                .progress_chars("=>-"),
            );
            bar
        };

        let changed_images: HashMap<Uuid, String> = HashMap::new();
        let changed_illustrations: HashMap<Uuid, String> = HashMap::new();
        let mut orphaned_images: Vec<Uuid> = Vec::new();
        let mut orphaned_illustrations: Vec<Uuid> = Vec::new();

        let card_futs: Vec<_> = cards
            .iter()
            .map(|info| self.upsert_card_info(info))
            .collect();
        let results: Vec<_> = futures::stream::iter(card_futs)
            .buffer_unordered(self.pool_size)
            .inspect(|_| {
                #[cfg(feature = "local-dev")]
                pb.inc(1);
            })
            .collect()
            .await;

        for (orphaned_img, orphaned_ill) in results {
            if let Some(id) = orphaned_img {
                orphaned_images.push(id);
            }
            if let Some(id) = orphaned_ill {
                orphaned_illustrations.push(id);
            }
        }

        #[cfg(feature = "local-dev")]
        pb.finish_with_message("done");

        let relation_futs: Vec<_> = cards
            .iter()
            .flat_map(|info| {
                let combos = info
                    .combos
                    .iter()
                    .map(|c| Either::Left(self.upsert_combo(c)));
                let tokens = info
                    .related_tokens
                    .iter()
                    .map(|t| Either::Right(self.upsert_related_token(t)));
                combos.chain(tokens)
            })
            .collect();
        futures::stream::iter(relation_futs)
            .buffer_unordered(self.pool_size)
            .collect::<Vec<_>>()
            .await;

        UpsertResult {
            changed_images,
            changed_illustrations,
            orphaned_images,
            orphaned_illustrations,
        }
    }

    async fn delete_orphaned_images(&self, ids: &[Uuid]) {
        log::info!("Deleting {} orphaned images", ids.len());

        for id in ids {
            if let Err(e) = sqlx::query(
                "DELETE FROM image WHERE id = $1
                 AND NOT EXISTS (SELECT 1 FROM card WHERE image_id = $1)",
            )
            .bind(id)
            .execute(&self.pool)
            .await
            {
                log::warn!("Failed to delete orphaned image {id}: {e}");
            }
        }
    }

    async fn delete_orphaned_illustrations(&self, ids: &[Uuid]) {
        log::info!("Deleting {} orphaned illustrations", ids.len());

        for id in ids {
            if let Err(e) = sqlx::query(
                "DELETE FROM illustration WHERE id = $1
                 AND NOT EXISTS (SELECT 1 FROM card WHERE illustration_id = $1)",
            )
            .bind(id)
            .execute(&self.pool)
            .await
            {
                log::warn!("Failed to delete orphaned illustration {id}: {e}");
            }
        }
    }
}
