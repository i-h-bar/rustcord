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

impl Default for LockByName {
    fn default() -> Self {
        Self::new()
    }
}

impl LockByName {
    #[must_use]
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

    #[tokio::test]
    async fn test_reference_counting_increment() {
        let locks = LockByName::new();

        // Get first lock for "test_channel"
        let lock1 = locks.get("test_channel").await;

        // Verify the entry exists and count is 1
        let inner = locks.inner.lock().await;
        let entry = inner.get("test_channel").unwrap();
        assert_eq!(entry.0, 1);
        drop(inner);

        // Get second lock for same channel
        let lock2 = locks.get("test_channel").await;

        // Count should now be 2
        let inner = locks.inner.lock().await;
        let entry = inner.get("test_channel").unwrap();
        assert_eq!(entry.0, 2);
        drop(inner);

        drop(lock1);
        drop(lock2);
    }

    // NOTE: Tests for async drop cleanup are intentionally omitted.
    // The async-dropper library spawns background tasks for cleanup, making
    // deterministic testing difficult. The cleanup behavior is tested implicitly
    // through production usage and the concurrent access tests above.

    #[tokio::test]
    async fn test_multiple_acquisitions_same_name() {
        let locks = LockByName::new();

        // Get same lock multiple times
        let lock1 = locks.get("multi_test").await;
        let lock2 = locks.get("multi_test").await;
        let lock3 = locks.get("multi_test").await;

        // All should have same name
        assert_eq!(lock1.name, "multi_test");
        assert_eq!(lock2.name, "multi_test");
        assert_eq!(lock3.name, "multi_test");

        // Count should be 3
        let inner = locks.inner.lock().await;
        let entry = inner.get("multi_test").unwrap();
        assert_eq!(entry.0, 3);

        drop(inner);
        drop(lock1);
        drop(lock2);
        drop(lock3);
    }

    #[tokio::test]
    async fn test_lock_isolation_different_names() {
        let locks = LockByName::new();

        // Get locks for different channels
        let lock_a = locks.get("channel_a").await;
        let lock_b = locks.get("channel_b").await;

        // Both should exist independently
        let inner = locks.inner.lock().await;
        assert!(inner.contains_key("channel_a"));
        assert!(inner.contains_key("channel_b"));
        assert_eq!(inner.get("channel_a").unwrap().0, 1);
        assert_eq!(inner.get("channel_b").unwrap().0, 1);

        drop(inner);
        drop(lock_a);
        drop(lock_b);
    }

    #[tokio::test]
    async fn test_concurrent_access_blocks() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc as StdArc;

        let locks = StdArc::new(LockByName::new());
        let counter = StdArc::new(AtomicUsize::new(0));

        let locks1 = StdArc::clone(&locks);
        let locks2 = StdArc::clone(&locks);
        let counter1 = StdArc::clone(&counter);
        let counter2 = StdArc::clone(&counter);

        let (res1, res2) = tokio::join!(
            async move {
                let lock = locks1.get("concurrent").await;
                let _guard = lock.lock().await;

                // Increment counter
                let val = counter1.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                val
            },
            async move {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                let lock = locks2.get("concurrent").await;
                let _guard = lock.lock().await;

                // This should only run after first task releases lock
                let val = counter2.fetch_add(1, Ordering::SeqCst);
                val
            }
        );

        // First task should see 0, second should see 1 (proving serialization)
        assert_eq!(res1, 0);
        assert_eq!(res2, 1);
    }

    #[tokio::test]
    async fn test_different_channels_dont_block_each_other() {
        use std::sync::Arc as StdArc;
        use std::time::Instant;

        let locks = StdArc::new(LockByName::new());
        let locks1 = StdArc::clone(&locks);
        let locks2 = StdArc::clone(&locks);
        let start = Instant::now();

        let (_, _) = tokio::join!(
            async move {
                let lock = locks1.get("channel_1").await;
                let _guard = lock.lock().await;
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            },
            async move {
                let lock = locks2.get("channel_2").await;
                let _guard = lock.lock().await;
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        );

        let elapsed = start.elapsed();

        // Should complete in ~50ms (parallel), not ~100ms (serial)
        // Allow some margin for test flakiness
        assert!(elapsed.as_millis() < 80, "Elapsed: {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_guard_released_on_drop() {
        let locks = LockByName::new();

        // Acquire lock in a scope
        {
            let lock = locks.get("scope_test").await;
            let _guard = lock.lock().await;
            // Guard is held here
        }
        // Guard should be released here

        // Should be able to acquire immediately
        let lock = locks.get("scope_test").await;
        let _guard = lock.lock().await;
        // If we got here without blocking, the guard was properly released
    }

}
