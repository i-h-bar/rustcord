use std::collections::HashSet;
use std::sync::LazyLock;
use tokio::sync::Mutex;

static LOCK: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub async fn wait_for_lock(channel_id: String) {
    for _ in 0..100 {
        {
            let mut active_channels = LOCK.lock().await;
            if !active_channels.contains(&channel_id) {
                active_channels.insert(channel_id);
                break;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

pub async fn remove_lock(channel_id: String) {
    LOCK.lock().await.remove(&channel_id);
}
