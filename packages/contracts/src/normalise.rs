use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

static PUNCTUATION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\w\s]").expect("Invalid regex"));

#[must_use]
pub fn normalise(name: &str) -> String {
    PUNCTUATION_RE
        .replace_all(&name.replace('-', " ").nfkc().collect::<String>(), "")
        .to_lowercase()
}