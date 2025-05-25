use std::collections::HashSet;
use std::sync::LazyLock;
use tokio::sync::Mutex;

static LOCK: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub async fn wait_for_lock(channel_id: String) {
    for _ in 0..100 {
        if !LOCK.lock().await.contains(&channel_id) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    LOCK.lock().await.insert(channel_id);
}

pub async fn remove_lock(channel_id: String) {
    LOCK.lock().await.remove(&channel_id);
}
