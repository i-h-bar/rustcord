use crate::dbs::psql::Psql;
use crate::mtg::card::FuzzyFound;
use sqlx::postgres::PgRow;
use sqlx::{Error, FromRow, Row};

pub const FUZZY_SEARCH_DISTINCT_CARDS: &str = r#"
select * from distinct_cards
where word_similarity(front_normalised_name, $1) > 0.50;
"#;

pub const FUZZY_SEARCH_SET_NAME: &str = r#"
select array_agg(normalised_name)
    from set
where word_similarity(normalised_name, $1) > 0.25
"#;

pub const FUZZY_SEARCH_ARTIST: &str = r#"
select array_agg(normalised_name)
    from artist
where word_similarity(normalised_name, $1) > 0.50
"#;

pub const NORMALISED_SET_NAME: &str = r#"select normalised_name from set where abbreviation = $1"#;

pub const RANDOM_CARD_FROM_DISTINCT: &str = r#"
            select set.id                       as set_id,
                   front.id                     as front_id,
                   front.name                   as front_name,
                   front.normalised_name        as front_normalised_name,
                   front.scryfall_url           as front_scryfall_url,
                   front.image_id               as front_image_id,
                   front.illustration_id        as front_illustration_id,
                   front_rule.mana_cost         as front_mana_cost,
                   front_rule.colour_identity   as front_colour_identity,
                   front_rule.power             as front_power,
                   front_rule.toughness         as front_toughness,
                   front_rule.loyalty           as front_loyalty,
                   front_rule.defence           as front_defence,
                   front_rule.type_line         as front_type_line,
                   front_rule.keywords          as front_keywords,
                   front_rule.oracle_text       as front_oracle_text,
            
                   back.id                      as back_id,
                   back.name                    as back_name,
                   back.scryfall_url            as back_scryfall_url,
                   back.image_id                as back_image_id,
                   back.illustration_id         as back_illustration_id,
                   back_rule.mana_cost          as back_mana_cost,
                   back_rule.colour_identity    as back_colour_identity,
                   back_rule.power              as back_power,
                   back_rule.toughness          as back_toughness,
                   back_rule.loyalty            as back_loyalty,
                   back_rule.defence            as back_defence,
                   back_rule.type_line          as back_type_line,
                   back_rule.keywords           as back_keywords,
                   back_rule.oracle_text        as back_oracle_text,
            
                   front.release_date           as release_date,
                   artist.name                  as artist,
                   set.name                     as set_name
            from card front
                     left join card back on front.backside_id = back.id
                     left join rule front_rule on front.rule_id = front_rule.id
                     left join rule back_rule on back.rule_id = back_rule.id
                     join set on front.set_id = set.id
                     left join artist on front.artist_id = artist.id
            where front.illustration_id is not null 
            order by random() 
            limit 1;
"#;

impl Psql {
    pub async fn fuzzy_search_distinct(&self, normalised_name: &str) -> Option<Vec<FuzzyFound>> {
        match sqlx::query(FUZZY_SEARCH_DISTINCT_CARDS)
            .bind(normalised_name)
            .fetch_all(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed fuzzy search distinct cards fetch - {why}");
                None
            }
            Ok(rows) => rows
                .into_iter()
                .map(|row| FuzzyFound::from_row(&row).ok())
                .collect(),
        }
    }

    pub async fn set_name_from_abbreviation(&self, abbreviation: &str) -> Option<String> {
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

    pub async fn fuzzy_search_set(
        &self,
        set_name: &str,
        normalised_name: &str,
    ) -> Option<Vec<FuzzyFound>> {
        match sqlx::query(&format!(
            r#"select * from set_{} where word_similarity(front_normalised_name, $1) > 0.50;"#,
            set_name.replace(" ", "_")
        ))
        .bind(normalised_name)
        .fetch_all(&self.pool)
        .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(rows) => rows
                .into_iter()
                .map(|row| FuzzyFound::from_row(&row).ok())
                .collect(),
        }
    }

    pub async fn fuzzy_search_set_name(&self, normalised_name: &str) -> Option<Vec<String>> {
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

    pub async fn fuzzy_search_for_artist(&self, normalised_name: &str) -> Option<Vec<String>> {
        match sqlx::query(FUZZY_SEARCH_ARTIST)
            .bind(normalised_name)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed artist fetch - {why}");
                None
            }
            Ok(row) => row.try_get::<Vec<String>, &str>("array_agg").ok(),
        }
    }

    pub async fn fuzzy_search_artist(
        &self,
        artist: &str,
        normalised_name: &str,
    ) -> Option<Vec<FuzzyFound>> {
        match sqlx::query(&format!(
            r#"select * from artist_{} where word_similarity(front_normalised_name, $1) > 0.50;"#,
            artist.replace(" ", "_")
        ))
        .bind(normalised_name)
        .fetch_all(&self.pool)
        .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(rows) => rows
                .into_iter()
                .map(|row| FuzzyFound::from_row(&row).ok())
                .collect(),
        }
    }

    pub async fn random_card(&self) -> Option<FuzzyFound> {
        match sqlx::query(RANDOM_CARD_FROM_DISTINCT)
            .fetch_one(&self.pool)
            .await
        {
            Err(why) => {
                log::warn!("Failed random card fetch - {why}");
                None
            }
            Ok(row) => FuzzyFound::from_row(&row).ok(),
        }
    }

    pub async fn random_card_from_set(&self, set_name: &str) -> Option<FuzzyFound> {
        match sqlx::query(&format!(
            r#"select * from set_{} where front_illustration_id is not null order by random() limit 1;"#,
            set_name.replace(" ", "_")
        ))
        .fetch_one(&self.pool)
        .await
        {
            Err(why) => {
                log::warn!("Failed search set fetch - {why}");
                None
            }
            Ok(row) => FuzzyFound::from_row(&row).ok(),
        }
    }
}

impl<'r> FromRow<'r, PgRow> for FuzzyFound {
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        FuzzyFound::from(row)
    }
}
