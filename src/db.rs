use once_cell::sync::OnceCell;
use std::env;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;


static DB_INSTANCE: OnceCell<PSQL> = OnceCell::new();

#[derive(Debug)]
pub struct PSQL {
    pub(crate) pool: Pool<Postgres>
}

impl PSQL {
    pub async fn init() {
        let instance = Self::new().await;
        DB_INSTANCE.set(instance).expect("Could not initialise DB once cell");
    }

    async fn new() -> Self {
        let uri = env::var("PSQL_URI").expect("Postgres uri wasn't in env vars");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&uri)
            .await
            .expect("Failed Postgres connection");

        Self { pool }
    }

    pub fn get() -> Option<&'static Self> {
        DB_INSTANCE.get()
    }
}