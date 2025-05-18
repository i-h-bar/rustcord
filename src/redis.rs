use redis::Client;
use std::env;
use redis::aio::MultiplexedConnection;
use tokio::sync::OnceCell;

static REDIS_INSTANCE: OnceCell<Redis> = OnceCell::const_new();

#[derive(Debug)]
pub struct Redis {
    connection: MultiplexedConnection
}

impl Redis {
    pub async fn init() {
        let redis = Self::new().await;
        REDIS_INSTANCE.set(redis).expect("Failed to init redis instance");
    }
    
    async fn new() -> Self {
        let url = env::var("REDIS_URL").expect("REDIS_URL must be set");
        let client = Client::open(url).expect("failed to open redis client");
        let connection = client.get_multiplexed_async_connection().await.expect("failed to get redis connection");
        
        Self { connection }
    }
    
    pub fn get(&self) -> Option<&'static Self> {
        REDIS_INSTANCE.get()
    }
}