use std::env;
use async_trait::async_trait;
use futures::future;
use sqlx::{Pool, Row};
use sqlx::postgres::PgPoolOptions;
use crate::ports::storage::{Set, Storage};

pub struct Postgres {
    pool: Pool<sqlx::Postgres>,
}

impl Postgres {
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
            Ok(result) => result.try_get::<i64, &str>("count").unwrap_or(0) as u32,
            Err(why) => {
                log::warn!("Failed to fetch volume: {}", why);
                0
            }
        }
    }
}

#[async_trait]
impl Storage for Postgres {
    async fn get_set_volumes(&self, sets: Vec<Set>) -> Vec<(Set, u32)> {
        future::join_all(
            sets.into_iter().map(|set| async {
                let volume = self.get_set_volume(&set).await;
                (set, volume)
            })
        ).await
    }
}