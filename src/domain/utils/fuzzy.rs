use std::cmp::Ordering;

pub trait ToBytes {
    fn to_bytes(&self) -> &[u8];
}

impl ToBytes for &str {
    fn to_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToBytes for String {
    fn to_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[allow(clippy::cast_precision_loss)]
pub fn jaro_winkler_ascii_bitmask<A: ToBytes + PartialEq<B>, B: ToBytes>(a: &A, b: &B) -> f32 {
    let a_chars = a.to_bytes();
    let b_chars = b.to_bytes();
    let len_a = a_chars.len();
    let len_b = b_chars.len();

    if a == b {
        return 1.0;
    }

    let max_dist = (len_a.max(len_b) / 2).saturating_sub(1);
    let mut matches = 0.0;
    let mut hash_a: u64 = 0;
    let mut hash_b: u64 = 0;

    for (i, a_char) in a_chars.iter().enumerate().take(len_a) {
        let start = i.saturating_sub(max_dist);
        let end = (i + max_dist + 1).min(len_b);

        for (j, b_char) in b_chars.iter().enumerate().take(end).skip(start) {
            if (hash_b & (1 << j)) == 0 && a_char == b_char {
                hash_a |= 1 << i;
                hash_b |= 1 << j;
                matches += 1.0;
                break;
            }
        }
    }

    if matches == 0.0 {
        return 0.0;
    }

    let mut transpositions = 0.0;
    let mut b_ptr = 0;

    for (i, &a_char) in a_chars.iter().enumerate().take(len_a) {
        if (hash_a & (1 << i)) != 0 {
            while (hash_b & (1 << b_ptr)) == 0 {
                b_ptr += 1;
            }
            if a_char != b_chars[b_ptr] {
                transpositions += 1.0;
            }
            b_ptr += 1;
        }
    }

    let jaro_similarity = (1.0 / 3.0)
        * (matches / len_a as f32
            + matches / len_b as f32
            + (matches - transpositions / 2.0) / matches);

    let mut prefix_len = 0;
    for (c1, c2) in a_chars.iter().zip(b_chars) {
        if c1 == c2 {
            prefix_len += 1;
        } else {
            break;
        }
    }

    #[allow(clippy::cast_sign_loss)]
    let prefix_len = (prefix_len as usize).min(4) as f32;
    let scaling_factor = 0.1;

    jaro_similarity + (prefix_len * scaling_factor * (1.0 - jaro_similarity))
}

pub fn winkliest_match<
    A: PartialEq<B> + ToBytes,
    B: ToBytes,
    I: AsRef<[B]> + IntoIterator<Item = B>,
>(
    target: &A,
    heap: I,
) -> Option<B> {
    let (_, closest_match) = heap
        .into_iter()
        .map(|needle| {
            let distance = jaro_winkler_ascii_bitmask(target, &needle);
            (distance, needle)
        })
        .max_by(|&(x, _), (y, _)| x.partial_cmp(y).unwrap_or(Ordering::Less))?;

    Some(closest_match)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaro_winkler_bitmasl() {
        let a = "CRATE";
        let b = "TRACE";

        let answer = jaro_winkler_ascii_bitmask(&a, &b);

        assert_eq!(answer, 0.73333335)
    }

    #[test]
    fn test_winkliest_match() {
        let a = "CRATE";
        let b = ["TRACE", "sdasda", "sadasdasd"];

        assert_eq!(winkliest_match(&a, b), Some("TRACE"))
    }

    #[test]
    fn test_winkliest_match_none() {
        let a = "CRATE";
        let b: [&str; 0] = [];

        assert_eq!(winkliest_match(&a, b), None)
    }

    #[test]
    fn test_jaro_winkler_identical_strings() {
        let a = "lightning bolt";
        let b = "lightning bolt";

        assert_eq!(jaro_winkler_ascii_bitmask(&a, &b), 1.0);
    }

    #[test]
    fn test_jaro_winkler_empty_strings() {
        let a = "";
        let b = "";

        assert_eq!(jaro_winkler_ascii_bitmask(&a, &b), 1.0);
    }

    #[test]
    fn test_jaro_winkler_one_empty_string() {
        let a = "card";
        let b = "";

        assert_eq!(jaro_winkler_ascii_bitmask(&a, &b), 0.0);
    }

    #[test]
    fn test_jaro_winkler_no_matches() {
        let a = "abc";
        let b = "xyz";

        assert_eq!(jaro_winkler_ascii_bitmask(&a, &b), 0.0);
    }

    #[test]
    fn test_jaro_winkler_threshold_boundary() {
        // Test around the 0.75 threshold used in the app
        let a = "gitrog monster";
        let b = "gitrog monstr";

        let score = jaro_winkler_ascii_bitmask(&a, &b);
        assert!(score > 0.75, "Expected score > 0.75, got {}", score);
    }

    #[test]
    fn test_jaro_winkler_case_sensitive() {
        // The algorithm is case-sensitive (normalization happens elsewhere)
        let a = "Lightning Bolt";
        let b = "lightning bolt";

        let score = jaro_winkler_ascii_bitmask(&a, &b);
        assert!(score < 1.0, "Should be case-sensitive");
    }

    #[test]
    fn test_jaro_winkler_long_strings() {
        let a = "the gitrog monster is a legendary frog horror creature";
        let b = "the gitrog monster is a legendary frog horor creature";

        let score = jaro_winkler_ascii_bitmask(&a, &b);
        assert!(score > 0.9, "Should handle long strings with minor typos");
    }

    #[test]
    fn test_jaro_winkler_single_char() {
        let a = "a";
        let b = "b";

        assert_eq!(jaro_winkler_ascii_bitmask(&a, &b), 0.0);
    }

    #[test]
    fn test_jaro_winkler_single_char_match() {
        let a = "x";
        let b = "x";

        assert_eq!(jaro_winkler_ascii_bitmask(&a, &b), 1.0);
    }

    #[test]
    fn test_jaro_winkler_prefix_boost() {
        // Strings with common prefix should score higher than those without
        let a = "lightning bolt";
        let with_prefix = "lightning strike";
        let without_prefix = "chain lightning";

        let score_with = jaro_winkler_ascii_bitmask(&a, &with_prefix);
        let score_without = jaro_winkler_ascii_bitmask(&a, &without_prefix);

        assert!(
            score_with > score_without,
            "Common prefix should boost score: {} vs {}",
            score_with,
            score_without
        );
    }

    #[test]
    fn test_jaro_winkler_transposition() {
        let a = "martha";
        let b = "marhta";

        let score = jaro_winkler_ascii_bitmask(&a, &b);
        assert!(score > 0.9, "Transpositions should still score high");
    }

    #[test]
    fn test_winkliest_match_card_names() {
        let target = "gitrog monster";
        let candidates = [
            "the gitrog monster",
            "gitrog monstre",
            "gideon of the trials",
        ];

        let result = winkliest_match(&target, candidates);
        // "gitrog monstre" is closer in length and has fewer differences
        assert_eq!(result, Some("gitrog monstre"));
    }

    #[test]
    fn test_winkliest_match_exact_substring() {
        // When exact match exists, it should win
        let target = "lightning bolt";
        let candidates = ["lightning bolt", "lightning strike", "chain lightning"];

        let result = winkliest_match(&target, candidates);
        assert_eq!(result, Some("lightning bolt"));
    }

    #[test]
    fn test_winkliest_match_with_typos() {
        let target = "lightnig bolt";
        let candidates = ["lightning bolt", "lightning strike", "chain lightning"];

        let result = winkliest_match(&target, candidates);
        assert_eq!(result, Some("lightning bolt"));
    }

    #[test]
    fn test_winkliest_match_tie_breaker() {
        // When scores are equal, should return first max found
        let target = "test";
        let candidates = ["test1", "test2"];

        let result = winkliest_match(&target, candidates);
        assert!(result.is_some());
    }

    #[test]
    fn test_to_bytes_trait_string() {
        let s = String::from("test");
        assert_eq!(s.to_bytes(), b"test");
    }

    #[test]
    fn test_to_bytes_trait_str() {
        let s = "test";
        assert_eq!(s.to_bytes(), b"test");
    }

    #[test]
    fn test_jaro_winkler_unicode_safe() {
        // Should handle UTF-8 safely (even if not ideal for non-ASCII)
        let a = "café";
        let b = "cafe";

        let score = jaro_winkler_ascii_bitmask(&a, &b);
        assert!(score > 0.0);
    }
}
