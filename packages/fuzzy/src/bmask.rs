use crate::ToBytes;

/// Scalar Jaro-Winkler over ASCII bytes (u64-bitmask implementation).
///
/// Kept public as the pure-scalar reference used by the benches and the
/// SIMD equivalence tests. For the fastest available implementation, use
/// [`crate::jaro_winkler_ascii_simd`] — it returns bit-identical scores.
#[allow(clippy::cast_precision_loss)]
pub fn jaro_winkler_ascii_bitmask<A: ToBytes, B: ToBytes>(a: &A, b: &B) -> f32 {
    // The u64 match masks can only track 64 positions; longer inputs
    // degrade to a comparison of their first 64 bytes.
    jaro_winkler_bytes(crate::truncate_bytes64(a), crate::truncate_bytes64(b))
}

#[inline]
#[allow(clippy::cast_precision_loss)]
pub(crate) fn jaro_winkler_bytes(a_chars: &[u8], b_chars: &[u8]) -> f32 {
    let len_a = a_chars.len();
    let len_b = b_chars.len();

    if a_chars == b_chars {
        return 1.0;
    }

    let max_dist = (len_a.max(len_b) / 2).saturating_sub(1);
    let mut matches: u32 = 0;
    let mut hash_a: u64 = 0;
    let mut hash_b: u64 = 0;

    for (i, a_char) in a_chars.iter().enumerate() {
        let end = (i + max_dist + 1).min(len_b);
        let start = i.saturating_sub(max_dist).min(end);

        for (j, b_char) in b_chars.iter().enumerate().take(end).skip(start) {
            if (hash_b & (1 << j)) == 0 && a_char == b_char {
                hash_a |= 1 << i;
                hash_b |= 1 << j;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut transpositions: u32 = 0;
    let mut b_matches = hash_b;

    for (i, &a_char) in a_chars.iter().enumerate() {
        if (hash_a & (1 << i)) != 0 {
            let j = b_matches.trailing_zeros() as usize;
            b_matches &= b_matches - 1;
            if a_char != b_chars[j] {
                transpositions += 1;
            }
        }
    }

    let matches = matches as f32;
    let jaro_similarity = (1.0 / 3.0)
        * (matches / len_a as f32
            + matches / len_b as f32
            + (matches - transpositions as f32 / 2.0) / matches);

    let prefix_len = a_chars
        .iter()
        .zip(b_chars)
        .take_while(|(c1, c2)| c1 == c2)
        .count()
        .min(4) as f32;

    jaro_similarity + (prefix_len * 0.1 * (1.0 - jaro_similarity))
}

#[cfg(test)]
mod tests {
    use super::jaro_winkler_ascii_bitmask;

    #[test]
    fn test_jaro_winkler_bitmask() {
        let a = "CRATE";
        let b = "TRACE";

        let answer = jaro_winkler_ascii_bitmask(&a, &b);

        assert_eq!(answer, 0.73333335)
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
    fn test_jaro_winkler_over_64_bytes_truncates() {
        // Inputs longer than 64 bytes are truncated to their first 64 bytes,
        // so strings identical up to byte 64 score as an exact match.
        let a = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffffgggg1234567890";
        let b = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffffgggg0987654321";
        assert_eq!(a.len(), 74);
        assert_eq!(b.len(), 74);

        assert_eq!(jaro_winkler_ascii_bitmask(&a, &b), 1.0);
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
