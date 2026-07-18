use crate::ToBytes;

#[cfg(target_arch = "x86_64")]
pub(crate) mod avx2 {
    use std::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
    };

    /// Mask with bits `[0, n)` set. `n` must be in `1..=64`.
    #[inline]
    fn low_bits(n: usize) -> u64 {
        debug_assert!((1..=64).contains(&n));
        u64::MAX >> (64 - n)
    }

    /// Combine two 32-lane byte-compare results into one 64-bit position mask.
    #[inline]
    #[target_feature(enable = "avx2")]
    // The i32 movemask result is a 32-bit lane bitmask, not a signed
    // quantity: reinterpreting its bits as u32 (then widening to u64) is
    // intentional, not a lossy numeric cast.
    #[allow(clippy::cast_sign_loss, clippy::cast_lossless)]
    fn movemask64(lo: __m256i, hi: __m256i) -> u64 {
        let lo = _mm256_movemask_epi8(lo) as u32 as u64;
        let hi = _mm256_movemask_epi8(hi) as u32 as u64;
        lo | (hi << 32)
    }

    /// AVX2 Jaro-Winkler over byte slices. Bit-identical to
    /// [`crate::jaro_winkler_bytes`].
    ///
    /// # Safety
    /// The caller must ensure the CPU supports AVX2 and that both slices are
    /// at most 64 bytes long.
    #[target_feature(enable = "avx2")]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
    pub(crate) unsafe fn jaro_winkler(a: &[u8], b: &[u8]) -> f32 {
        debug_assert!(a.len() <= 64 && b.len() <= 64);
        let len_a = a.len();
        let len_b = b.len();

        if len_a == 0 && len_b == 0 {
            return 1.0;
        }
        if len_a == 0 || len_b == 0 {
            return 0.0;
        }

        // Zero-padded copies so each whole string fits in two YMM registers.
        let mut a_buf = [0u8; 64];
        let mut b_buf = [0u8; 64];
        a_buf[..len_a].copy_from_slice(a);
        b_buf[..len_b].copy_from_slice(b);

        // SAFETY: both buffers are 64 bytes; loadu has no alignment requirement.
        let (a_lo, a_hi, b_lo, b_hi) = unsafe {
            (
                _mm256_loadu_si256(a_buf.as_ptr().cast()),
                _mm256_loadu_si256(a_buf.as_ptr().add(32).cast()),
                _mm256_loadu_si256(b_buf.as_ptr().cast()),
                _mm256_loadu_si256(b_buf.as_ptr().add(32).cast()),
            )
        };

        // Positions where a and b agree byte-for-byte (padding included).
        let eq_mask = movemask64(_mm256_cmpeq_epi8(a_lo, b_lo), _mm256_cmpeq_epi8(a_hi, b_hi));
        if len_a == len_b && eq_mask == u64::MAX {
            return 1.0;
        }

        let max_dist = (len_a.max(len_b) / 2).saturating_sub(1);
        let b_valid = low_bits(len_b);

        let mut matches: u32 = 0;
        let mut hash_a: u64 = 0;
        let mut hash_b: u64 = 0;

        for i in 0..len_a {
            // SAFETY: i < len_a <= 64.
            let c = unsafe { *a_buf.get_unchecked(i) };
            let needle = _mm256_set1_epi8(c as i8);
            let eq = movemask64(
                _mm256_cmpeq_epi8(needle, b_lo),
                _mm256_cmpeq_epi8(needle, b_hi),
            );

            let start = i.saturating_sub(max_dist);
            let window = (u64::MAX << start) & low_bits((i + max_dist + 1).min(64)) & b_valid;

            let candidates = eq & window & !hash_b;
            if candidates != 0 {
                hash_a |= 1 << i;
                // Greedy first fit: lowest candidate bit, same as the scalar loop.
                hash_b |= candidates & candidates.wrapping_neg();
                matches += 1;
            }
        }

        if matches == 0 {
            return 0.0;
        }

        let mut transpositions: u32 = 0;
        let mut a_matches = hash_a;
        let mut b_matches = hash_b;
        while a_matches != 0 {
            let i = a_matches.trailing_zeros() as usize;
            let j = b_matches.trailing_zeros() as usize;
            a_matches &= a_matches - 1;
            b_matches &= b_matches - 1;
            // SAFETY: i and j are bit positions of u64 masks, so < 64.
            if unsafe { a_buf.get_unchecked(i) != b_buf.get_unchecked(j) } {
                transpositions += 1;
            }
        }

        // Common prefix from the equality mask, clamped to the shorter length
        // so zero padding can never extend it past real bytes.
        let neq = !eq_mask | !low_bits(len_a.min(len_b));
        let prefix_len = neq.trailing_zeros().min(4) as f32;

        let matches = matches as f32;
        let jaro_similarity = (1.0 / 3.0)
            * (matches / len_a as f32
                + matches / len_b as f32
                + (matches - transpositions as f32 / 2.0) / matches);

        let scaling_factor = 0.1;
        jaro_similarity + (prefix_len * scaling_factor * (1.0 - jaro_similarity))
    }
}

/// Jaro-Winkler over ASCII bytes, dispatching to an AVX2 kernel when the CPU
/// supports it and falling back to the scalar bitmask core otherwise.
///
/// Scores are bit-identical to [`crate::jaro_winkler_ascii_bitmask`]; inputs
/// longer than 64 bytes are truncated to their first 64 bytes.
#[must_use]
pub fn jaro_winkler_ascii_simd<A: ToBytes, B: ToBytes>(a: &A, b: &B) -> f32 {
    jaro_winkler_slices(a.to_bytes(), b.to_bytes())
}

/// Slice-level entry point for the same dispatch: safe for slices of any
/// length (truncates to 64 bytes internally before the kernel).
#[must_use]
pub(crate) fn jaro_winkler_slices(a: &[u8], b: &[u8]) -> f32 {
    let a_chars = &a[..a.len().min(64)];
    let b_chars = &b[..b.len().min(64)];

    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") {
        // SAFETY: AVX2 availability just checked; slices truncated to <= 64 bytes above.
        return unsafe { avx2::jaro_winkler(a_chars, b_chars) };
    }

    crate::jaro_winkler_bytes(a_chars, b_chars)
}

#[cfg(test)]
mod tests {
    use super::jaro_winkler_ascii_simd;

    // Deterministic pseudo-random corpus (no dev-dependency needed).
    fn lcg_corpus() -> Vec<String> {
        let mut state: u64 = 0x243F_6A88_85A3_08D3;
        let alphabet = b"abcdefghijklmnopqrstuvwxyz 0123456789";
        let mut out = Vec::new();
        for len in [0usize, 1, 2, 5, 11, 18, 31, 57, 63, 64, 70, 100] {
            for _ in 0..8 {
                let s: String = (0..len)
                    .map(|_| {
                        state = state
                            .wrapping_mul(6_364_136_223_846_793_005)
                            .wrapping_add(1_442_695_040_888_963_407);
                        char::from(alphabet[(state >> 33) as usize % alphabet.len()])
                    })
                    .collect();
                out.push(s);
            }
        }
        out
    }

    #[test]
    fn test_simd_matches_bitmask_on_corpus() {
        let corpus = lcg_corpus();
        for a in &corpus {
            for b in &corpus {
                let expected = crate::jaro_winkler_ascii_bitmask(a, b);
                let actual = jaro_winkler_ascii_simd(a, b);
                assert_eq!(actual, expected, "mismatch for {a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn test_simd_known_score() {
        // Same fixture as the bitmask test suite: exact score must match.
        assert_eq!(jaro_winkler_ascii_simd(&"CRATE", &"TRACE"), 0.733_333_35);
    }

    #[test]
    fn test_simd_identical() {
        assert_eq!(
            jaro_winkler_ascii_simd(&"lightning bolt", &"lightning bolt"),
            1.0
        );
    }

    #[test]
    fn test_simd_empty_both() {
        assert_eq!(jaro_winkler_ascii_simd(&"", &""), 1.0);
    }

    #[test]
    fn test_simd_one_empty() {
        assert_eq!(jaro_winkler_ascii_simd(&"card", &""), 0.0);
    }

    #[test]
    fn test_simd_no_matches() {
        assert_eq!(jaro_winkler_ascii_simd(&"abc", &"xyz"), 0.0);
    }

    #[test]
    fn test_simd_over_64_bytes_truncates() {
        let a = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffffgggg1234567890";
        let b = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffffgggg0987654321";
        assert_eq!(jaro_winkler_ascii_simd(&a, &b), 1.0);
    }

    #[test]
    fn test_simd_embedded_nul_matches_bitmask() {
        // NUL never occurs in normalised input, but the two implementations
        // must still agree so equivalence is unconditional.
        let a = "ab\0";
        let b = "ab";
        assert_eq!(
            jaro_winkler_ascii_simd(&a, &b),
            crate::jaro_winkler_ascii_bitmask(&a, &b)
        );
    }

    #[test]
    fn test_simd_accepts_string_type() {
        let a = String::from("black lotus");
        let b = String::from("black lotos");
        assert!(jaro_winkler_ascii_simd(&a, &b) > 0.9);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_avx2_kernel_directly() {
        if !is_x86_feature_detected!("avx2") {
            eprintln!("skipping: AVX2 not available on this CPU");
            return;
        }
        // SAFETY: AVX2 availability checked above; inputs are <= 64 bytes.
        let score = unsafe { super::avx2::jaro_winkler(b"CRATE", b"TRACE") };
        assert_eq!(score, 0.733_333_35);
    }
}
