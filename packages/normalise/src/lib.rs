use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

static PUNCTUATION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\w\s]").expect("Invalid regex"));

#[must_use]
pub fn normalise_card_name(name: &str) -> String {
    PUNCTUATION_RE
        .replace_all(&name.replace('-', " ").nfkc().collect::<String>(), "")
        .to_lowercase()
}

#[must_use]
pub fn normalise_emoji_name(name: &str) -> String {
    let normalised: String = name
        .nfkc()
        .collect::<String>()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();

    if normalised.len() < 2 {
        format!("{normalised}_")
    } else {
        normalised
    }
}
