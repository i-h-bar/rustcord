use crate::adapters::drivers::discord::emoji::discord::get_emoji;
use crate::domain::utils::REGEX_COLLECTION;
use contracts::card::Card;
use contracts::emoji::normalise_name;
use lru::LruCache;
use serenity::all::ReactionType;
use std::num::NonZeroUsize;
use std::sync::{LazyLock, Mutex};

static NORMALISED_NAME_CACHE: LazyLock<Mutex<LruCache<String, String>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(256).unwrap())));

fn cached_normalise_name(symbol: &str) -> String {
    let mut cache = NORMALISED_NAME_CACHE.lock().unwrap();
    if let Some(name) = cache.get(symbol) {
        return name.clone();
    }

    let name = normalise_name(symbol);
    cache.put(symbol.to_string(), name.clone());
    name
}

fn canonical_pair(identity: &str) -> Option<&'static str> {
    match identity {
        "WU" | "UW" => Some("W/U"),
        "WB" | "BW" => Some("W/B"),
        "WR" | "RW" => Some("R/W"),
        "WG" | "GW" => Some("G/W"),
        "UB" | "BU" => Some("U/B"),
        "UR" | "RU" => Some("U/R"),
        "UG" | "GU" => Some("G/U"),
        "BR" | "RB" => Some("B/R"),
        "BG" | "GB" => Some("B/G"),
        "RG" | "GR" => Some("R/G"),
        _ => None,
    }
}

async fn symbol_reaction(symbol: &str, fallback: ReactionType) -> ReactionType {
    let name = cached_normalise_name(&format!("{{{symbol}}}"));
    match get_emoji(&name).await {
        Some(emoji) => ReactionType::Custom {
            animated: false,
            id: emoji.id,
            name: Some(emoji.name),
        },
        None => fallback,
    }
}

pub async fn colour_id_emoji(card: &Card) -> ReactionType {
    let identity = card.colour_identity().join("");

    if identity.is_empty() {
        return symbol_reaction("C", ReactionType::Unicode("✨".to_string())).await;
    }

    let base = match identity.len() {
        1 => Some(identity.clone()),
        2 => canonical_pair(&identity).map(str::to_string),
        5 => return ReactionType::Unicode("🌈".to_string()),
        _ => None,
    };

    let Some(base) = base else {
        return ReactionType::Unicode("✨".to_string());
    };

    let symbol = if card.mana_cost().contains("/P") {
        format!("{base}/P")
    } else {
        base
    };

    symbol_reaction(&symbol, ReactionType::Unicode("✨".to_string())).await
}

async fn symbol_markdown(symbol: &str) -> String {
    let name = cached_normalise_name(symbol);
    match get_emoji(&name).await {
        Some(emoji) => format!("<:{}:{}>", emoji.name, emoji.id),
        None => symbol.to_string(),
    }
}

pub async fn add_emoji(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for m in REGEX_COLLECTION.symbols.find_iter(text) {
        result.push_str(&text[last_end..m.start()]);
        result.push_str(&symbol_markdown(m.as_str()).await);
        last_end = m.end();
    }
    result.push_str(&text[last_end..]);

    result
}
