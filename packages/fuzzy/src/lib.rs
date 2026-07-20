use std::cmp::Ordering;

mod bmask;
mod simd;

pub use bmask::jaro_winkler_ascii_bitmask;
pub use simd::jaro_winkler_ascii_simd;

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

/// AVX2/scalar dispatch, skipping the length truncation `jaro_winkler_ascii_simd` does.
///
/// # Safety
/// `a` and `b` must each be at most 64 bytes. Violating this panics on the
/// AVX2 path (bounds-checked buffer copy), but on the scalar fallback in a
/// release build it silently returns a wrong score instead of panicking —
/// the match-bit shift (`1 << i`) overflows a `u64` and wraps once
/// overflow checks are off.
#[inline]
pub(crate) unsafe fn jaro_winkler_unchecked(a: &[u8], b: &[u8]) -> f32 {
    debug_assert!(a.len() <= 64 && b.len() <= 64);

    #[cfg(all(not(feature = "avx2"), target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") {
        // SAFETY: AVX2 availability just checked; caller guarantees length <= 64.
        return unsafe { simd::avx2::jaro_winkler(a, b) };
    }

    #[cfg(feature = "avx2")]
    // SAFETY: the `avx2` feature asserts AVX2 is available; caller guarantees length <= 64.
    return unsafe { simd::avx2::jaro_winkler(a, b) };

    #[cfg(not(feature = "avx2"))]
    bmask::jaro_winkler_bytes(a, b)
}

/// Return the candidate from `heap` with the highest Jaro-Winkler similarity
/// to `target`, or `None` if `heap` is empty.
///
/// Scoring uses the same accelerated dispatch as [`jaro_winkler_ascii_simd`].
pub fn winkliest_match<A: ToBytes, B: ToBytes, I: IntoIterator<Item = B>>(
    target: &A,
    heap: I,
) -> Option<B> {
    let target_bytes_checked = {
        let target_bytes = target.to_bytes();
        &target_bytes[..target_bytes.len().min(64)]
    };
    let (_, closest_match) = heap
        .into_iter()
        .map(|needle| {
            let needle_bytes_checked = {
                let needle_bytes = needle.to_bytes();
                &needle_bytes[..needle_bytes.len().min(64)]
            };

            let distance =
                unsafe { jaro_winkler_unchecked(target_bytes_checked, needle_bytes_checked) };
            (distance, needle)
        })
        .max_by(|&(x, _), (y, _)| x.partial_cmp(y).unwrap_or(Ordering::Less))?;

    Some(closest_match)
}

/// Sort candidates by descending Jaro-Winkler similarity to `target`.
///
/// The relative order of equal-scored candidates is unspecified.
/// Scoring uses the same accelerated dispatch as [`jaro_winkler_ascii_simd`].
pub fn winkliest_sort<A: ToBytes, B: ToBytes, I: IntoIterator<Item = B>>(
    target: &A,
    heap: I,
) -> Vec<B> {
    let target_bytes_checked = {
        let target_bytes = target.to_bytes();
        &target_bytes[..target_bytes.len().min(64)]
    };
    let mut scored: Vec<_> = heap
        .into_iter()
        .map(|needle| {
            let needle_bytes_checked = {
                let needle_bytes = needle.to_bytes();
                &needle_bytes[..needle_bytes.len().min(64)]
            };

            (
                unsafe { jaro_winkler_unchecked(target_bytes_checked, needle_bytes_checked) },
                needle,
            )
        })
        .collect();

    scored.sort_unstable_by(|(x, _), (y, _)| y.partial_cmp(x).unwrap_or(Ordering::Equal));
    scored.into_iter().map(|(_, item)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaro_winkler_bitmask() {
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
    fn test_winkliest_sort_orders_by_score() {
        let target = "lightning bolt";
        let candidates = ["chain lightning", "lightning strike", "lightning bolt"];

        let result = winkliest_sort(&target, candidates);
        assert_eq!(result[0], "lightning bolt");
    }

    #[test]
    fn test_winkliest_sort_empty() {
        let target = "lightning bolt";
        let candidates: [&str; 0] = [];

        let result = winkliest_sort(&target, candidates);
        assert!(result.is_empty());
    }

    #[test]
    fn test_winkliest_sort_preserves_all_elements() {
        let target = "gitrog monster";
        let candidates = [
            "the gitrog monster",
            "gitrog monstre",
            "gideon of the trials",
        ];

        let result = winkliest_sort(&target, candidates);
        assert_eq!(result.len(), 3);
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
    fn test_winkliest_match_accepts_plain_iterator() {
        // Iterator adapters (here: filter) implement IntoIterator but not
        // AsRef<[&str]>, so this only compiles once the dead bound is gone.
        let a = "CRATE";
        let candidates = ["TRACE", "zzz", "sdasda"];

        let result = winkliest_match(&a, candidates.into_iter().filter(|s| s.len() > 3));

        assert_eq!(result, Some("TRACE"));
    }

    #[test]
    fn test_jaro_winkler_unicode_safe() {
        // Should handle UTF-8 safely (even if not ideal for non-ASCII)
        let a = "café";
        let b = "cafe";

        let score = jaro_winkler_ascii_bitmask(&a, &b);
        assert!(score > 0.0);
    }

    #[test]
    fn test_winkliest_sort_order_matches_bitmask_scores() {
        // Pinning test: winkliest_sort's ordering must always agree with
        // sorting by jaro_winkler_ascii_bitmask scores directly. Candidates
        // are chosen with strictly distinct scores so the assertion is
        // insensitive to tie-ordering.
        let target = "lightning bolt";
        let candidates = [
            "chain lightning",
            "lightning strike",
            "lightning bolt",
            "bolt",
        ];

        let sorted = winkliest_sort(&target, candidates);

        let mut expected: Vec<(f32, &str)> = candidates
            .iter()
            .map(|c| (jaro_winkler_ascii_bitmask(&target, c), *c))
            .collect();
        expected.sort_by(|(x, _), (y, _)| y.partial_cmp(x).unwrap_or(Ordering::Equal));
        let expected: Vec<&str> = expected.into_iter().map(|(_, c)| c).collect();

        assert_eq!(sorted, expected);
    }
}
