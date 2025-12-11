mod redis;

use crate::adapters::outbound::cache::redis::Redis;
use crate::ports::outbound::cache::Cache;

#[must_use]
pub fn init_cache() -> impl Cache {
    Redis::create()
}
