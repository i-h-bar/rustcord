use crate::ports::emoji::{Emoji, EmojiMetaData, EmojiStore};
use async_trait::async_trait;
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use reqwest::Client;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::env;
use std::num::NonZeroU32;
use std::sync::Arc;

type Limiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

#[derive(Serialize, Deserialize)]
struct DiscordEmojiList {
    items: Vec<EmojiMetaData>,
}

pub struct Discord {
    client: Client,
    base_url: String,
    app_id: String,
    limiter: Arc<Limiter>,
}

impl Discord {
    pub fn new() -> Self {
        let token = env::var("BOT_TOKEN").expect("BOT_TOKEN is not env vars");
        let app_id = env::var("APPLICATION_ID").expect("APPLICATION_ID is not env vars");
        let base_url =
            env::var("BASE_DISCORD_URL").unwrap_or("https://discord.com/api/v10".to_string());
        let user_agent = env::var("USER_AGENT").expect("USER_AGENT wasn't in env vars");

        let mut headers = HeaderMap::new();
        let auth_value = HeaderValue::from_str(&format!("Bot {token}"))
            .expect("Invalid BOT_TOKEN value for auth header");
        headers.insert(AUTHORIZATION, auth_value);

        let client = Client::builder()
            .user_agent(user_agent)
            .default_headers(headers)
            .build()
            .expect("Failure to creating reqwest client for discord");

        let quota = Quota::per_second(NonZeroU32::new(50).expect("50 quota is unavailable"));
        let limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            client,
            base_url,
            app_id,
            limiter,
        }
    }
}

#[async_trait]
impl EmojiStore for Discord {
    async fn get_emojis(&self) -> Vec<EmojiMetaData> {
        let url = format!("{}/applications/{}/emojis", self.base_url, self.app_id);

        self.limiter.until_ready().await;
        let resp = self.client.get(url).send().await.unwrap();

        let emojis: DiscordEmojiList = resp.json().await.unwrap();

        emojis.items
    }

    async fn upload_emojis(&self, emojis: Vec<Emoji>) {
        todo!()
    }
}
