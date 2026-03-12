mod redis;

use crate::adapters::services::cache::redis::Redis;
use crate::ports::services::cache::Cache;

#[must_use]
pub fn init_cache() -> impl Cache {
    Redis::create()
}
