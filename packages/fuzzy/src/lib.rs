//! Fast Jaro-Winkler string similarity over ASCII bytes.
//!
//! This crate scores similarity between short, already-normalised strings —
//! card names, set names, artist names — for fuzzy search and guess matching.
//! It is not a general-purpose Unicode similarity library; see the assumptions
//! below.
//!
//! # Input assumptions
//!
//! - **Pre-normalised ASCII.** Scoring compares bytes, so it is case- and
//!   accent-sensitive. Callers are expected to have lower-cased and stripped
//!   diacritics beforehand (`"Café"` and `"cafe"` do *not* score as equal).
//! - **First 64 bytes only.** Every entry point truncates each input to its
//!   first 64 bytes before scoring — the algorithm tracks match positions in a
//!   `u64` bitmask. Strings that are identical up to byte 64 score as an exact
//!   match. This is ample for the card/set/artist names this crate targets.
//!
//! # Public API
//!
//! - [`jaro_winkler_ascii_simd`] — the fast path; dispatches to an AVX2 kernel
//!   when available and falls back to the scalar implementation otherwise.
//!   Prefer this for scoring a single pair.
//! - [`jaro_winkler_ascii_bitmask`] — the pure scalar reference. Returns
//!   scores bit-identical to the SIMD path; kept public for benches and tests.
//! - [`winkliest_match`] / [`winkliest_sort`] — pick the closest candidate, or
//!   order a set of candidates, against one target. Both use the same
//!   accelerated dispatch.
//! - [`ToBytes`] — the trait inputs implement to expose their bytes. Provided
//!   for `&str` and `String`; implement it on a newtype to score other types.
//!
//! # The `avx2` feature
//!
//! By default the SIMD path performs a (cached) runtime `is_x86_feature_detected!`
//! check per process and stays portable across `x86_64` CPUs. Enabling the
//! `avx2` feature **asserts** AVX2 is present and skips that check, calling the
//! AVX2 kernel unconditionally. Only enable it when the deployment CPU is known
//! to support AVX2 — a binary built with the feature will execute an illegal
//! instruction on a CPU without it. The feature is rejected at compile time on
//! non-`x86_64` targets.

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
/// If several candidates tie for the highest score, the last one in iteration
/// order is returned. Scoring uses the same accelerated dispatch as
/// [`jaro_winkler_ascii_simd`].
#[must_use]
pub fn winkliest_match<A: ToBytes, B: ToBytes, I: IntoIterator<Item = B>>(
    target: &A,
    heap: I,
) -> Option<B> {
    let target_bytes_checked = truncate_bytes64(target);
    let (_, closest_match) = heap
        .into_iter()
        .map(|needle| {
            let needle_bytes_checked = truncate_bytes64(&needle);

            // SAFETY: truncated to <= 64 bytes above.
            (
                unsafe { jaro_winkler_unchecked(target_bytes_checked, needle_bytes_checked) },
                needle,
            )
        })
        .max_by(|&(x, _), (y, _)| x.partial_cmp(y).unwrap_or(Ordering::Less))?;

    Some(closest_match)
}

/// Sort candidates by descending Jaro-Winkler similarity to `target`.
///
/// The relative order of equal-scored candidates is unspecified.
/// Scoring uses the same accelerated dispatch as [`jaro_winkler_ascii_simd`].
#[must_use]
pub fn winkliest_sort<A: ToBytes, B: ToBytes, I: IntoIterator<Item = B>>(
    target: &A,
    heap: I,
) -> Vec<B> {
    let target_bytes_checked = truncate_bytes64(target);
    let mut scored: Vec<_> = heap
        .into_iter()
        .map(|needle| {
            let needle_bytes_checked = truncate_bytes64(&needle);

            (
                // SAFETY: truncated to <= 64 bytes above.
                unsafe { jaro_winkler_unchecked(target_bytes_checked, needle_bytes_checked) },
                needle,
            )
        })
        .collect();

    scored.sort_unstable_by(|(x, _), (y, _)| y.partial_cmp(x).unwrap_or(Ordering::Less));
    scored.into_iter().map(|(_, item)| item).collect()
}

/// Truncate a value's bytes to the first 64 (the length the kernels can score).
#[must_use]
#[inline]
pub(crate) fn truncate_bytes64(target: &impl ToBytes) -> &[u8] {
    truncate64(target.to_bytes())
}

/// Truncate a byte slice to its first 64 bytes.
#[must_use]
#[inline]
pub(crate) fn truncate64(target: &[u8]) -> &[u8] {
    &target[..target.len().min(64)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn jaro_winkler_unchecked_debug_asserts_length_contract() {
        // The unsafe contract (each slice <= 64 bytes) is guarded by a
        // debug_assert that fires before any unchecked access. Callers only
        // reach the kernel through `truncate_bytes64`, so this cannot happen in
        // practice — this test is the tripwire if that guard ever regresses.
        let over = [b'a'; 65];
        // SAFETY: intentionally violating the length precondition to prove the
        // debug_assert catches it; in a debug (test) build the assert panics
        // before the kernel performs any unsafe read.
        let _ = unsafe { jaro_winkler_unchecked(&over, &over) };
    }

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
        // Two candidates that score identically against the target: each is
        // "test" plus one trailing digit that matches nothing in "test", so
        // the scores tie. `max_by` returns the last equal-max element, so the
        // later candidate in iteration order must win.
        let target = "test";
        let first = "test1";
        let last = "test2";

        // Premise: the two candidates really do tie.
        assert_eq!(
            jaro_winkler_ascii_simd(&target, &first),
            jaro_winkler_ascii_simd(&target, &last),
            "candidates must tie for this to exercise tie-breaking",
        );

        // The last equal-max candidate wins...
        assert_eq!(winkliest_match(&target, [first, last]), Some(last));
        // ...and swapping the order flips which one wins, confirming the
        // choice is by iteration order, not by value.
        assert_eq!(winkliest_match(&target, [last, first]), Some(first));
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
