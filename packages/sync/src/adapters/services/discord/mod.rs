use crate::domain::utils::emoji::normalise_name;
use crate::domain::utils::svg;
use crate::ports::emoji::{
    EmojiImage, EmojiMetaData, EmojiName, EmojiStore, SetEmoji, SymbolEmoji,
};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use futures::future;
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

#[derive(Serialize, Deserialize)]
struct EmojiUpload {
    name: String,
    image: String,
}

impl EmojiUpload {
    pub fn new(name: String, image: &str) -> Self {
        let image = format!("data:image/png;base64,{image}");
        Self { name, image }
    }
}

impl From<SetEmoji> for EmojiUpload {
    fn from(emoji: SetEmoji) -> Self {
        let name = normalise_name(&emoji.name);

        EmojiUpload::new(name, &emoji.image.0)
    }
}

impl From<SymbolEmoji> for EmojiUpload {
    fn from(emoji: SymbolEmoji) -> Self {
        let name = normalise_name(&emoji.name);

        EmojiUpload::new(name, &emoji.image.0)
    }
}

pub struct Discord {
    client: Client,
    base_url: String,
    app_id: String,
    limiter: Arc<Limiter>,
}

impl Default for Discord {
    fn default() -> Self {
        Self::new()
    }
}

impl Discord {
    /// # Panics
    /// Panics if any of the required environment variables are not set (`BOT_TOKEN`, `USER_AGENT`,
    /// `APPLICATION_ID`), if `BOT_TOKEN` contains characters invalid for an HTTP header value, or
    /// if the reqwest client fails to build.
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

        let quota = Quota::per_second(NonZeroU32::new(5).expect("5 quota is unavailable"));
        let limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            client,
            base_url,
            app_id,
            limiter,
        }
    }

    async fn upload_emoji(&self, emoji: impl Into<EmojiUpload> + EmojiName) {
        let url = format!("{}/applications/{}/emojis", self.base_url, self.app_id);
        let name = emoji.clone_name();
        self.limiter.until_ready().await;

        let resp = match self
            .client
            .post(&url)
            .json::<EmojiUpload>(&emoji.into())
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(why) => {
                log::warn!("Unable to upload emoji {name}: {why}");
                return;
            }
        };

        if !resp.status().is_success() {
            let status_code = resp.status();
            log::warn!("None success status code for emoji {name}: {status_code}");
            if status_code == reqwest::StatusCode::TOO_MANY_REQUESTS {
                std::process::exit(1);
            }
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

    async fn upload_set_symbols(&self, emojis: Vec<SetEmoji>) {
        if emojis.is_empty() {
            return;
        }

        log::info!("Uploading Set Symbols");
        future::join_all(emojis.iter().map(|s| async {
            let recoloured = svg::recolour(&s.image.0, "#C9A227");
            let Some(png) = svg::to_png(&recoloured) else {
                return;
            };

            let emoji = SetEmoji {
                name: s.name.clone(),
                image: EmojiImage(STANDARD.encode(&png)),
            };

            self.upload_emoji(emoji).await;
        }))
        .await;
    }

    async fn upload_symbol_emojis(&self, emojis: Vec<SymbolEmoji>) {
        if emojis.is_empty() {
            return;
        }

        log::info!("Uploading Emojis");
        future::join_all(emojis.iter().map(|emoji| async {
            let Some(png) = svg::to_png(&emoji.image.0) else {
                return;
            };

            let emoji = SetEmoji {
                name: emoji.name.clone(),
                image: EmojiImage(STANDARD.encode(&png)),
            };

            self.upload_emoji(emoji).await;
        }))
        .await;
    }
}
