use serenity::all::{Emoji, Http};
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};
use tokio::sync::{OnceCell, RwLock};

const SYNC_COOLDOWN: Duration = Duration::from_hours(4);

static EMOJI_CACHE: OnceCell<DiscordEmojiCache> = OnceCell::const_new();

struct DiscordEmojiCache {
    cache: RwLock<HashMap<String, Emoji>>,
    last_sync: RwLock<Instant>,
    http: Http,
}

impl DiscordEmojiCache {
    async fn new() -> Self {
        let token = env::var("BOT_TOKEN").expect("Bot token wasn't in env vars");
        let http = Http::new(&token);

        match http.get_current_application_info().await {
            Ok(info) => http.set_application_id(info.id),
            Err(why) => log::warn!("Failed to fetch application info: {why}"),
        }

        let cache = RwLock::new(HashMap::with_capacity(2000));
        let last_sync = RwLock::new(
            Instant::now()
                .checked_sub(SYNC_COOLDOWN)
                .unwrap_or(Instant::now())
                .checked_sub(Duration::from_mins(1))
                .unwrap_or(Instant::now()),
        );
        let obj = Self {
            cache,
            last_sync,
            http,
        };
        obj.sync().await;
        obj
    }

    async fn sync(&self) {
        if self.last_sync.read().await.elapsed() < SYNC_COOLDOWN {
            return;
        }

        let emojis = match self.http.get_application_emojis().await {
            Ok(emoji) => emoji,
            Err(why) => {
                log::warn!("Failure to fetch emoji {why}");
                return;
            }
        };

        // Replace all entries so stale IDs (from deleted/re-uploaded emojis) are corrected.
        let mut cache = self.cache.write().await;
        for emoji in emojis {
            cache.insert(emoji.name.clone(), emoji);
        }

        *self.last_sync.write().await = Instant::now();
    }

    async fn get(&self, name: &str) -> Option<Emoji> {
        if let Some(emoji) = self.cache.read().await.get(name).cloned() {
            return Some(emoji);
        }

        self.sync().await;

        self.cache.read().await.get(name).cloned()
    }
}

pub async fn warmup_emoji() {
    EMOJI_CACHE.get_or_init(DiscordEmojiCache::new).await;
}

pub async fn get_emoji(name: &str) -> Option<Emoji> {
    let cache = EMOJI_CACHE.get_or_init(DiscordEmojiCache::new).await;
    cache.get(name).await
}
