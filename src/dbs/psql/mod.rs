use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use tokio::sync::OnceCell;

pub mod queries;

static PSQL: OnceCell<Psql> = OnceCell::const_new();

#[derive(Debug)]
pub struct Psql {
    pub(crate) pool: Pool<Postgres>,
}

impl Psql {
    pub async fn init() {
        let instance = Self::new().await;
        PSQL
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
        PSQL.get()
    }
}
