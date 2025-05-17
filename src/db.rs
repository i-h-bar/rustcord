use once_cell::sync::OnceCell;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;

static DB_INSTANCE: OnceCell<Psql> = OnceCell::new();

#[derive(Debug)]
pub struct Psql {
    pub(crate) pool: Pool<Postgres>,
}

impl Psql {
    pub async fn init() {
        let instance = Self::new().await;
        DB_INSTANCE
            .set(instance)
            .expect("Could not initialise DB once cell");
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
