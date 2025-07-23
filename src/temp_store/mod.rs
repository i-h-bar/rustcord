mod redis;

use async_trait::async_trait;

#[async_trait]
trait TempStore {
    type Key;
    type Value;
    
    fn new() -> Self;
    async fn get(&self, key: Self::Key) -> Option<Self::Value>;
    async fn set(&self, key: Self::Key, value: Self::Value) -> Result<(), anyhow::Error>;
    async fn delete(&self, key: Self::Key)  -> Result<(), anyhow::Error>;
}
