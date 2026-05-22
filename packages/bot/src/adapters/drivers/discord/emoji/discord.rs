use std::collections::HashMap;
use std::env;
use serenity::all::{Emoji, Http};
use tokio::sync::{OnceCell, RwLock};

static EMOJI_CACHE: OnceCell<DiscordEmojiCache> = OnceCell::const_new();

struct DiscordEmojiCache {
    cache: RwLock<HashMap<String, Emoji>>,
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

        let cache = RwLock::new(HashMap::with_capacity(1000));
        let obj = Self { cache, http };
        obj.sync().await;
        obj
    }

    async fn sync(&self) {
        let emojis = match self.http.get_application_emojis().await {
            Ok(emoji) => emoji,
            Err(why) => {
                log::warn!("Failure to fetch emoji {why}");
                return;
            },
        };

        let new_emojis: Vec<Emoji> = {
            let cache = self.cache.read().await;
            emojis.into_iter().filter(|e| !cache.contains_key(&e.name)).collect()
        };

        if new_emojis.is_empty() {
            return;
        }

        let mut cache = self.cache.write().await;
        for emoji in new_emojis {
            cache.insert(emoji.name.clone(), emoji);
        }
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