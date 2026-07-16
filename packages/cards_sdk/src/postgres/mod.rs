mod queries;

#[cfg(feature = "local-dev")]
use indicatif::{ProgressBar, ProgressStyle};

use crate::ids::{ChannelId, GuildId, SubscriptionId};
use crate::ingest::{
    Artist, CardInfo, CardRecord, Combo, Illustration, Image, Legality, Price, RelatedToken, Rule,
    Set, UpsertResult,
};
use crate::postgres::queries::{
    ALL_PRINTS, CARD_FROM_ID, FUZZY_SEARCH_CARD_AND_ARTIST, FUZZY_SEARCH_CARD_AND_SET_NAME,
    FUZZY_SEARCH_DISTINCT_CARDS, FUZZY_SEARCH_SET_NAME, NORMALISED_SET_NAME, PENDING_CARDS,
    RANDOM_CARD, RANDOM_SET_CARD, SIMILAR_CARDS_FROM, SUBSCRIPTION_EXISTS,
    SUBSCRIPTIONS_WITH_PENDING,
};
use crate::repository::{ReadRepository, SpoilerQueue, WriteRepository};
use crate::spoiler::{PendingCard, Subscription};
use async_trait::async_trait;
use contracts::card::Card;
use contracts::card_set::CardSet;
use futures::StreamExt;
use futures::future::Either;
use sqlx::postgres::{PgConnection, PgPoolOptions, PgRow};
use sqlx::types::time::Date;
use sqlx::{Connection, Pool, Row, error::DatabaseError};
use std::env;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

const DEFAULT_MAX_CONNECTIONS: u32 = 5;

/// A `spoiler_queue` row older than this is purged by `prune_queue`
/// regardless of delivery state — a safety valve for a subscription whose
/// webhook silently died and will never advance its cursor again, which
/// would otherwise block the cursor-based half of `prune_queue` forever.
const MAX_QUEUE_AGE_DAYS: i64 = 7;

/// Shared advisory lock key used by `create_for_batch` to serialize the
/// "check usage, then connect" step across concurrently-starting instances,
/// avoiding a race where two instances both see the same headroom and both
/// try to claim it.
const POOL_SIZING_LOCK_KEY: i64 = 727_663_001;

pub struct Postgres {
    pool: Pool<sqlx::Postgres>,
    pool_size: usize,
}

impl Postgres {
    /// # Panics
    /// Panics if `POSTGRES_USER`, `POSTGRES_PW`, or `POSTGRES_DB` env vars are not set,
    /// or if the connection to Postgres cannot be established.
    pub async fn create() -> Self {
        Self::connect_with_max_connections(&connection_uri(), DEFAULT_MAX_CONNECTIONS).await
    }

    /// For batch workloads (e.g. `sync`'s bulk/spoiler commands) that want to
    /// use as much of the available Postgres connection budget as it can
    /// safely spare, rather than a small fixed pool. Reserves
    /// `reserve_for_others` connections for other long-lived consumers (e.g.
    /// `bot`) and claims the rest.
    ///
    /// # Panics
    /// Panics under the same conditions as `create`, and if the initial probe
    /// connection or its queries fail.
    pub async fn create_for_batch(reserve_for_others: u32) -> Self {
        let uri = connection_uri();

        let mut probe = PgConnection::connect(&uri)
            .await
            .expect("Failed Postgres connection");

        sqlx::query("SELECT pg_advisory_lock($1)")
            .bind(POOL_SIZING_LOCK_KEY)
            .execute(&mut probe)
            .await
            .expect("Failed to acquire pool-sizing advisory lock");

        let max_connections: String = sqlx::query_scalar("SHOW max_connections")
            .fetch_one(&mut probe)
            .await
            .expect("Failed to read max_connections");
        let max_connections: u32 = max_connections
            .trim()
            .parse()
            .expect("max_connections wasn't a valid number");

        let in_use: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_stat_activity WHERE pid != pg_backend_pid()",
        )
        .fetch_one(&mut probe)
        .await
        .expect("Failed to read pg_stat_activity");

        sqlx::query("SELECT pg_advisory_unlock($1)")
            .bind(POOL_SIZING_LOCK_KEY)
            .execute(&mut probe)
            .await
            .expect("Failed to release pool-sizing advisory lock");

        probe.close().await.ok();

        let in_use = u32::try_from(in_use).unwrap_or(max_connections);
        let available = max_connections
            .saturating_sub(in_use)
            .saturating_sub(reserve_for_others)
            .max(1);

        log::info!(
            "Sizing batch pool: max_connections={max_connections}, in_use={in_use}, \
             reserved={reserve_for_others} -> using {available}"
        );

        Self::connect_with_max_connections(&uri, available).await
    }

    async fn connect_with_max_connections(uri: &str, max_connections: u32) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(uri)
            .await
            .expect("Failed Postgres connection");

        Self {
            pool,
            pool_size: max_connections as usize,
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

    async fn upsert_card(&self, card: &CardRecord) -> (Option<Uuid>, Option<Uuid>, bool) {
        match sqlx::query_as::<_, (Option<Uuid>, Option<Uuid>, bool)>(
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
               (SELECT illustration_id FROM prev) AS prev_illustration_id,
               (SELECT 1 FROM prev) IS NULL AS is_new",
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
            Ok(Some((prev_img, prev_ill, is_new))) => {
                let orphaned_img = prev_img.filter(|&old| old != card.image_id);
                let orphaned_ill = prev_ill.filter(|old| Some(*old) != card.illustration_id);
                (orphaned_img, orphaned_ill, is_new)
            }
            Ok(None) => (None, None, false),
            Err(e) => {
                log::warn!("Failed to upsert card {}: {}", card.id, e);
                (None, None, false)
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

    async fn upsert_card_info(&self, info: &CardInfo) -> (Option<Uuid>, Option<Uuid>, bool) {
        self.upsert_artist(&info.artist).await;
        self.upsert_image(&info.image).await;
        if let Some(ill) = &info.illustration {
            self.upsert_illustration(ill).await;
        }
        self.upsert_set(&info.set).await;
        self.upsert_rule(&info.rule).await;
        self.upsert_legality(&info.legality).await;
        let (orphaned_img, orphaned_ill, is_new) = self.upsert_card(&info.card).await;
        self.upsert_price(&info.price).await;

        (orphaned_img, orphaned_ill, is_new)
    }
}

#[async_trait]
impl ReadRepository for Postgres {
    async fn search(&self, normalised_name: &str) -> Option<Vec<Card>> {
        match sqlx::query(FUZZY_SEARCH_DISTINCT_CARDS)
            .bind(normalised_name)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed fuzzy search distinct cards fetch - {why}");
                None
            }
            Ok(rows) => Some(rows.into_iter().map(|row| card_from(&row)).collect()),
        }
    }

    async fn search_artist(&self, artist: &str, normalised_name: &str) -> Option<Vec<Card>> {
        match sqlx::query(FUZZY_SEARCH_CARD_AND_ARTIST)
            .bind(normalised_name)
            .bind(artist)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(rows) => Some(rows.into_iter().map(|row| card_from(&row)).collect()),
        }
    }

    async fn search_set(&self, set_name: &str, normalised_name: &str) -> Option<Vec<Card>> {
        match sqlx::query(FUZZY_SEARCH_CARD_AND_SET_NAME)
            .bind(normalised_name)
            .bind(set_name)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(rows) => Some(rows.into_iter().map(|row| card_from(&row)).collect()),
        }
    }

    async fn search_for_set_name(&self, normalised_name: &str) -> Option<Vec<String>> {
        match sqlx::query(FUZZY_SEARCH_SET_NAME)
            .bind(normalised_name)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed set name fetch - {why}");
                None
            }
            Ok(row) => row.try_get::<Vec<String>, &str>("array_agg").ok(),
        }
    }

    async fn set_name_from_abbreviation(&self, abbreviation: &str) -> Option<String> {
        match sqlx::query(NORMALISED_SET_NAME)
            .bind(abbreviation)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed set name from abbr fetch - {why}");
                None
            }
            Ok(row) => Some(row.get::<String, &str>("normalised_name")),
        }
    }

    async fn random_card(&self) -> Option<Card> {
        match sqlx::query(RANDOM_CARD).fetch_one(&self.pool).await {
            Err(why) => {
                log::warn!("Failed random card fetch - {why}");
                None
            }
            Ok(row) => Some(card_from(&row)),
        }
    }

    async fn random_card_from_set(&self, set_name: &str) -> Option<Card> {
        match sqlx::query(RANDOM_SET_CARD)
            .bind(set_name)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(row) => Some(card_from(&row)),
        }
    }

    async fn all_prints(&self, oracle_id: &Uuid) -> Option<Vec<CardSet>> {
        match sqlx::query(ALL_PRINTS)
            .bind(oracle_id)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed search all prints fetch - {why}");
                None
            }
            Ok(rows) => Some(rows.into_iter().map(|row| set_from(&row)).collect()),
        }
    }

    async fn fetch_card_by_id(&self, id: &Uuid) -> Option<Card> {
        match sqlx::query(CARD_FROM_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed card fetch - {why}");
                None
            }
            Ok(row) => Some(card_from(&row)),
        }
    }

    async fn similar_cards(&self, card: &Card) -> Option<Vec<Card>> {
        match sqlx::query(SIMILAR_CARDS_FROM)
            .bind(card.normalised_name())
            .bind(card.oracle_id())
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed search all prints fetch - {why}");
                None
            }
            Ok(rows) => Some(rows.into_iter().map(|row| card_from(&row)).collect()),
        }
    }
}

#[async_trait]
impl WriteRepository for Postgres {
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

        let mut orphaned_images: Vec<Uuid> = Vec::new();
        let mut orphaned_illustrations: Vec<Uuid> = Vec::new();
        let mut new_card_ids: Vec<Uuid> = Vec::new();

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

        for (info, (orphaned_img, orphaned_ill, is_new)) in cards.iter().zip(results) {
            if let Some(id) = orphaned_img {
                orphaned_images.push(id);
            }
            if let Some(id) = orphaned_ill {
                orphaned_illustrations.push(id);
            }
            if is_new {
                new_card_ids.push(info.card.id);
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

        if !new_card_ids.is_empty() {
            if let Err(e) =
                sqlx::query("INSERT INTO spoiler_queue (card_id) SELECT * FROM UNNEST($1::uuid[])")
                    .bind(&new_card_ids)
                    .execute(&self.pool)
                    .await
            {
                log::warn!(
                    "Failed to enqueue {} new card(s) for spoiler delivery: {e}",
                    new_card_ids.len()
                );
            } else {
                log::info!(
                    "Enqueued {} new card(s) for spoiler delivery",
                    new_card_ids.len()
                );
            }
        }

        UpsertResult {
            orphaned_images,
            orphaned_illustrations,
        }
    }

    async fn delete_orphaned_images(&self, ids: &[Uuid]) -> Vec<Uuid> {
        let mut deleted = Vec::new();
        for id in ids {
            match sqlx::query(
                "DELETE FROM image WHERE id = $1
                 AND NOT EXISTS (SELECT 1 FROM card WHERE image_id = $1)",
            )
            .bind(id)
            .execute(&self.pool)
            .await
            {
                Ok(result) if result.rows_affected() > 0 => deleted.push(*id),
                Ok(_) => {}
                Err(e) => log::warn!("Failed to delete orphaned image {id}: {e}"),
            }
        }
        deleted
    }

    async fn delete_orphaned_illustrations(&self, ids: &[Uuid]) -> Vec<Uuid> {
        let mut deleted = Vec::new();
        for id in ids {
            match sqlx::query(
                "DELETE FROM illustration WHERE id = $1
                 AND NOT EXISTS (SELECT 1 FROM card WHERE illustration_id = $1)",
            )
            .bind(id)
            .execute(&self.pool)
            .await
            {
                Ok(result) if result.rows_affected() > 0 => deleted.push(*id),
                Ok(_) => {}
                Err(e) => log::warn!("Failed to delete orphaned illustration {id}: {e}"),
            }
        }
        deleted
    }
}

#[async_trait]
impl SpoilerQueue for Postgres {
    async fn subscriptions_with_pending(&self) -> Vec<Subscription> {
        match sqlx::query(SUBSCRIPTIONS_WITH_PENDING)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed to fetch subscriptions with pending cards: {why}");
                Vec::new()
            }
            Ok(rows) => rows
                .into_iter()
                .map(|row| Subscription {
                    guild_id: row.get::<GuildId, &str>("guild_id"),
                    channel_id: row.get::<ChannelId, &str>("channel_id"),
                    id: row.get::<SubscriptionId, &str>("webhook_id"),
                    token: row.get::<String, &str>("webhook_token"),
                    cursor: row.get::<i64, &str>("cursor"),
                })
                .collect(),
        }
    }

    async fn subscription_exists(&self, guild_id: GuildId, channel_id: ChannelId) -> bool {
        match sqlx::query_as::<_, (bool,)>(SUBSCRIPTION_EXISTS)
            .bind(guild_id)
            .bind(channel_id)
            .fetch_one(&self.pool)
            .await
        {
            Ok((exists,)) => exists,
            Err(e) => {
                log::warn!(
                    "Failed to check for an existing subscription for guild {guild_id} channel {channel_id}: {e}"
                );
                false
            }
        }
    }

    async fn pending_cards(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
        limit: i64,
    ) -> Vec<PendingCard> {
        match sqlx::query(PENDING_CARDS)
            .bind(guild_id)
            .bind(channel_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!(
                    "Failed to fetch pending cards for guild {guild_id} channel {channel_id}: {why}"
                );
                Vec::new()
            }
            Ok(rows) => rows
                .into_iter()
                .map(|row| PendingCard {
                    queue_id: row.get::<i64, &str>("queue_id"),
                    card: card_from(&row),
                })
                .collect(),
        }
    }

    async fn ack(&self, guild_id: GuildId, channel_id: ChannelId, up_to_queue_id: i64) {
        if let Err(e) = sqlx::query(
            "UPDATE spoiler_subscription
             SET cursor = $3, consecutive_failures = 0
             WHERE guild_id = $1 AND channel_id = $2 AND cursor < $3",
        )
        .bind(guild_id)
        .bind(channel_id)
        .bind(up_to_queue_id)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to advance cursor for guild {guild_id} channel {channel_id}: {e}");
        }
    }

    async fn create_subscription(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
        webhook_id: SubscriptionId,
        webhook_token: &str,
    ) {
        if let Err(e) = sqlx::query(
            "INSERT INTO spoiler_subscription
               (guild_id, channel_id, webhook_id, webhook_token, cursor)
             VALUES ($1, $2, $3, $4, COALESCE((SELECT max(id) FROM spoiler_queue), 0))
             ON CONFLICT (guild_id, channel_id) DO UPDATE SET
               webhook_id    = EXCLUDED.webhook_id,
               webhook_token = EXCLUDED.webhook_token",
        )
        .bind(guild_id)
        .bind(channel_id)
        .bind(webhook_id)
        .bind(webhook_token)
        .execute(&self.pool)
        .await
        {
            log::warn!(
                "Failed to create spoiler subscription for guild {guild_id} channel {channel_id}: {e}"
            );
        }
    }

    async fn delete_subscription(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Option<SubscriptionId> {
        match sqlx::query_as::<_, (SubscriptionId,)>(
            "DELETE FROM spoiler_subscription
             WHERE guild_id = $1 AND channel_id = $2
             RETURNING webhook_id",
        )
        .bind(guild_id)
        .bind(channel_id)
        .fetch_optional(&self.pool)
        .await
        {
            Ok(row) => row.map(|(webhook_id,)| webhook_id),
            Err(e) => {
                log::warn!(
                    "Failed to delete spoiler subscription for guild {guild_id} channel {channel_id}: {e}"
                );
                None
            }
        }
    }

    async fn record_failure(&self, guild_id: GuildId, channel_id: ChannelId) -> i64 {
        match sqlx::query_as::<_, (i32,)>(
            "UPDATE spoiler_subscription
             SET consecutive_failures = consecutive_failures + 1
             WHERE guild_id = $1 AND channel_id = $2
             RETURNING consecutive_failures",
        )
        .bind(guild_id)
        .bind(channel_id)
        .fetch_optional(&self.pool)
        .await
        {
            Ok(Some((count,))) => i64::from(count),
            Ok(None) => 0,
            Err(e) => {
                log::warn!(
                    "Failed to record delivery failure for guild {guild_id} channel {channel_id}: {e}"
                );
                0
            }
        }
    }

    async fn prune_queue(&self) {
        // A subscription's `cursor` is the id of the last row *delivered* to
        // it (see `ack`/`pending_cards`, which excludes `id <= cursor` from
        // future work) — so a row is safe to delete once `id <= min(cursor)`
        // across all subscriptions, not merely `<`. `min(cursor)` over zero
        // subscriptions is NULL, and `id <= NULL` is never true — so the
        // first arm of this OR is a no-op (not a full-table delete) when
        // nobody is subscribed yet, which is the safe direction to fail in.
        // The second arm (age cap) still applies independently in that case,
        // so rows don't accumulate forever even with zero subscribers.
        let cutoff = OffsetDateTime::now_utc() - Duration::days(MAX_QUEUE_AGE_DAYS);

        match sqlx::query(
            "DELETE FROM spoiler_queue
             WHERE id <= (SELECT min(cursor) FROM spoiler_subscription)
                OR detected_at < $1",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await
        {
            Ok(result) if result.rows_affected() > 0 => {
                log::info!(
                    "Pruned {} spoiler_queue row(s) (delivered to all subscribers, or older than {MAX_QUEUE_AGE_DAYS} days)",
                    result.rows_affected()
                );
            }
            Ok(_) => {}
            Err(e) => log::warn!("Failed to prune spoiler_queue: {e}"),
        }
    }
}

fn set_from(row: &PgRow) -> CardSet {
    CardSet::new(
        row.get::<Uuid, &str>("card_id"),
        row.get::<String, &str>("set_name"),
        row.get::<&str, &str>("set_abbreviation"),
        row.get::<Date, &str>("release_date"),
    )
}

fn connection_uri() -> String {
    let user = env::var("POSTGRES_USER").expect("POSTGRES_USER wasn't in env vars");
    let password = env::var("POSTGRES_PW").expect("POSTGRES_PW wasn't in env vars");
    let db = env::var("POSTGRES_DB").expect("POSTGRES_DB wasn't in env vars");
    let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost:5432".to_string());
    format!("postgresql://{user}:{password}@{host}/{db}")
}

fn card_from(row: &PgRow) -> Card {
    Card::new(
        row.get::<Uuid, &str>("front_id"),
        row.get::<String, &str>("front_name"),
        row.get::<String, &str>("front_normalised_name"),
        row.get::<Uuid, &str>("front_oracle_id"),
        row.get::<String, &str>("front_scryfall_url"),
        row.get::<Uuid, &str>("front_image_id"),
        row.get::<Option<Uuid>, &str>("front_illustration_id"),
        row.get::<String, &str>("front_mana_cost"),
        row.get::<Vec<String>, &str>("front_colour_identity"),
        row.get::<Option<String>, &str>("front_power"),
        row.get::<Option<String>, &str>("front_toughness"),
        row.get::<Option<String>, &str>("front_loyalty"),
        row.get::<Option<String>, &str>("front_defence"),
        row.get::<String, &str>("front_type_line"),
        row.get::<String, &str>("front_oracle_text"),
        row.get::<Option<Uuid>, &str>("back_id"),
        row.get::<String, &str>("artist"),
        row.get::<String, &str>("set_name"),
        row.get::<String, &str>("set_abbreviation"),
        row.get::<Date, &str>("release_date"),
    )
}
