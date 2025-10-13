pub mod fuzzy;
pub mod mutex;

use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

const CARD_QUERY_RE: &str = r"(?i)\[\[(.*?)(:?(?:\s)*\|(?:\s)*(:?set(?:\s)*=(?:\s)*(.*?)?)?)?(:?(?:\s)*\|(?:\s)*(:?artist(?:\s)*=(?:\s)*(.*?)?)?)?]]";
const SYMBOL_RE: &str = r"(\{T}|\{Q}|\{E}|\{P}|\{PW}|\{CHAOS}|\{A}|\{TK}|\{X}|\{Y}|\{Z}|\{0}|\{Â½}|\{1}|\{2}|\{3}|\{4}|\{5}|\{6}|\{7}|\{8}|\{9}|\{10}|\{11}|\{12}|\{13}|\{14}|\{15}|\{16}|\{17}|\{18}|\{19}|\{20}|\{100}|\{1000000}|\{âˆž}|\{W/U}|\{W/B}|\{B/R}|\{B/G}|\{U/B}|\{U/R}|\{R/G}|\{R/W}|\{G/W}|\{G/U}|\{B/G/P}|\{B/R/P}|\{G/U/P}|\{G/W/P}|\{R/G/P}|\{R/W/P}|\{U/B/P}|\{U/R/P}|\{W/B/P}|\{W/U/P}|\{C/W}|\{C/U}|\{C/B}|\{C/R}|\{C/G}|\{2/W}|\{2/U}|\{2/B}|\{2/R}|\{2/G}|\{H}|\{W/P}|\{U/P}|\{B/P}|\{R/P}|\{G/P}|\{C/P}|\{HW}|\{HR}|\{W}|\{U}|\{B}|\{R}|\{G}|\{C}|\{S}|\{L}|\{D})";
const REMINDER_TEXT: &str = r"\((.+)\)";

pub static REGEX_COLLECTION: LazyLock<RegexCollection> = LazyLock::new(|| {
    let punctuation_removal = Regex::new(r"[^\w\s]").expect("Invalid regex");
    let cards = Regex::new(CARD_QUERY_RE).expect("Invalid regex");
    let symbols = Regex::new(SYMBOL_RE).expect("Invalid regex");
    let reminder_text = Regex::new(REMINDER_TEXT).expect("Invalid regex");
    RegexCollection {
        punctuation_removal,
        cards,
        symbols,
        reminder_text,
    }
});

pub struct RegexCollection {
    pub punctuation_removal: Regex,
    pub cards: Regex,
    pub symbols: Regex,
    pub reminder_text: Regex,
}

#[must_use]
pub fn normalise(name: &str) -> String {
    REGEX_COLLECTION
        .punctuation_removal
        .replace_all(&name.replace('-', " ").nfkc().collect::<String>(), "")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalise_simple_string() {
        assert_eq!(normalise("Lightning Bolt"), "lightning bolt");
    }

    #[test]
    fn test_normalise_with_punctuation() {
        assert_eq!(normalise("Jace, the Mind Sculptor"), "jace the mind sculptor");
    }

    #[test]
    fn test_normalise_with_apostrophe() {
        assert_eq!(normalise("Ajani's Pridemate"), "ajanis pridemate");
    }

    #[test]
    fn test_normalise_with_hyphen() {
        // Hyphens are replaced with spaces
        assert_eq!(normalise("X-Men"), "x men");
    }

    #[test]
    fn test_normalise_with_multiple_hyphens() {
        assert_eq!(normalise("Alpha-Beta-Gamma"), "alpha beta gamma");
    }

    #[test]
    fn test_normalise_unicode_characters() {
        // NFKC normalization handles some but not all Unicode
        // Æ remains as æ (lowercase)
        assert_eq!(normalise("Ætherling"), "ætherling");
    }

    #[test]
    fn test_normalise_already_normalized() {
        assert_eq!(normalise("already normalized"), "already normalized");
    }

    #[test]
    fn test_normalise_empty_string() {
        assert_eq!(normalise(""), "");
    }

    #[test]
    fn test_normalise_only_punctuation() {
        // ! is not matched by \w so it remains
        assert_eq!(normalise("!!!"), "");
    }

    #[test]
    fn test_normalise_mixed_case() {
        assert_eq!(normalise("ThE GiTrOg MoNsTeR"), "the gitrog monster");
    }

    #[test]
    fn test_normalise_numbers() {
        assert_eq!(normalise("Time Walk 123"), "time walk 123");
    }

    #[test]
    fn test_normalise_special_characters() {
        // ™ is removed but becomes two chars when converted
        assert_eq!(normalise("Urza's Saga™"), "urzas sagatm");
    }

    #[test]
    fn test_normalise_multiple_spaces() {
        // Multiple spaces should be preserved (not collapsed)
        assert_eq!(normalise("Multiple  Spaces"), "multiple  spaces");
    }

    #[test]
    fn test_normalise_leading_trailing_spaces() {
        assert_eq!(normalise("  Padded  "), "  padded  ");
    }

    #[test]
    fn test_normalise_parentheses() {
        assert_eq!(normalise("Card (Name)"), "card name");
    }

    #[test]
    fn test_normalise_brackets() {
        assert_eq!(normalise("[Card] Name"), "card name");
    }

    #[test]
    fn test_normalise_slashes() {
        assert_eq!(normalise("Life/Death"), "lifedeath");
    }

    #[test]
    fn test_normalise_underscores() {
        assert_eq!(normalise("Card_Name"), "card_name");
    }

    #[test]
    fn test_normalise_dots() {
        assert_eq!(normalise("Dr. Doom"), "dr doom");
    }

    #[test]
    fn test_normalise_real_card_names() {
        assert_eq!(normalise("Dack Fayden"), "dack fayden");
        assert_eq!(normalise("Jace, Vryn's Prodigy"), "jace vryns prodigy");
        assert_eq!(normalise("Emrakul, the Aeons Torn"), "emrakul the aeons torn");
    }

    #[test]
    fn test_normalise_japanese_characters() {
        // Should handle Unicode normalization
        let result = normalise("カード");
        assert!(result.chars().all(|c| !c.is_ascii_punctuation()));
    }

    #[test]
    fn test_normalise_accented_characters() {
        // NFKC doesn't decompose all accents
        assert_eq!(normalise("Café"), "café");
        assert_eq!(normalise("Señor"), "señor");
    }

    #[test]
    fn test_normalise_ampersand() {
        assert_eq!(normalise("Rock & Roll"), "rock  roll");
    }

    #[test]
    fn test_normalise_idempotent() {
        let input = "Jace, the Mind Sculptor";
        let first = normalise(input);
        let second = normalise(&first);
        assert_eq!(first, second);
    }
}
