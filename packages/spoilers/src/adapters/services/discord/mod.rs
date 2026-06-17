use std::env;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use crate::ports::emoji::{Emoji, EmojiMetaData, EmojiStore};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
struct DiscordEmojiList {
    items: Vec<EmojiMetaData>,
}


pub struct Discord {
    client: Client,
    base_url: String,
    app_id: String,
}

impl Discord {
    pub fn new() -> Self {
        let token = env::var("BOT_TOKEN").expect("BOT_TOKEN is not env vars");
        let app_id = env::var("APPLICATION_ID").expect("APPLICATION_ID is not env vars");
        let base_url = env::var("BASE_DISCORD_URL").unwrap_or("https://discord.com/api/v10".to_string());
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

        Self {
            client,
            base_url,
            app_id,
        }
    }
}


#[async_trait]
impl EmojiStore for Discord {
    async fn get_emojis(&self) -> Vec<EmojiMetaData> {
        let url = format!("{}/applications/{}/emojis", self.base_url, self.app_id);
        let resp = self.client.get(url).send().await.unwrap();


        let emojis: DiscordEmojiList = resp.json().await.unwrap();

        emojis.items
    }

    async fn upload_emojis(&self, emojis: Vec<Emoji>) {
        todo!()
    }
}