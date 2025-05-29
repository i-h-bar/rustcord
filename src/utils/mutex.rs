use async_dropper::AsyncDrop;
use serenity::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::sync::{Mutex, MutexGuard};

pub static LOCKS: LazyLock<LockByName> = LazyLock::new(LockByName::new);

type Inner = HashMap<String, (usize, Arc<Mutex<()>>)>;

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

impl LockByName {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get(&self, name: &str) -> NamedLock {
        let m = {
            let mut lock = self.inner.lock().await;
            let entry = lock
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
        if let Some(entry) = lock.get_mut(&self.name) {
            entry.0 -= 1;
            if entry.0 == 0 {
                lock.remove(&self.name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lock_by_name() {
        let lock = LOCKS.get("name").await;
        assert_eq!(lock.name, "name");
        let _guard = lock.lock().await;
        let another_lock = LOCKS.get("name2").await;
        assert_eq!(another_lock.name, "name2");
        let _guard_2 = another_lock.lock().await;
    }

    #[tokio::test]
    async fn test_lock() {
        let (Ok(a), Ok(b)) = tokio::join!(
            tokio::spawn(async move {
                let lock = LOCKS.get("name").await;
                let _guard = lock.lock().await;
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                1
            }),
            tokio::spawn(async move {
                let lock = LOCKS.get("name").await;
                let _guard = lock.lock().await;
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                2
            }),
        ) else {
            panic!("test went wrong")
        };

        assert_eq!(a, 1);
        assert_eq!(b, 2);
    }
}
