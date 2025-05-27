use async_dropper::AsyncDrop;
use serenity::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::sync::{Mutex, MutexGuard};

pub static LOCKS: LazyLock<LockByName> = LazyLock::new(LockByName::new);

pub struct LockByName {
    inner: Arc<Mutex<Inner>>,
}
pub struct NamedLock {
    name: String,
    inner: Arc<Mutex<Inner>>,
    lock: Arc<Mutex<()>>,
}
pub struct NamedGuard<'a> {
    _guard: MutexGuard<'a, ()>,
}

struct Inner {
    map: HashMap<String, (usize, Arc<Mutex<()>>)>,
}

impl LockByName {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                map: HashMap::new(),
            })),
        }
    }

    pub async fn get(&self, name: &str) -> NamedLock {
        let m = {
            let mut lock = self.inner.lock().await;
            let entry = lock
                .map
                .entry(name.to_string())
                .or_insert_with(|| (0, Arc::new(Mutex::new(()))));
            entry.0 += 1;
            Arc::clone(&entry.1)
        };

        NamedLock {
            name: name.to_string(),
            inner: Arc::clone(&self.inner),
            lock: m,
        }
    }
}

impl NamedLock {
    pub async fn lock(&self) -> NamedGuard<'_> {
        NamedGuard {
            _guard: self.lock.lock().await,
        }
    }
}

#[async_trait]
impl AsyncDrop for NamedLock {
    async fn async_drop(&mut self) {
        let mut lock = self.inner.lock().await;
        let entry = lock.map.get_mut(&self.name).unwrap();
        entry.0 -= 1;
        if entry.0 == 0 {
            lock.map.remove(&self.name);
        }
    }
}
