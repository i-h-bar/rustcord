use std::cmp::Ordering;
use std::io::Bytes;
use std::str::Chars;

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for &str {
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl ToBytes for String {
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
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

    for i in 0..len_a {
        let start = i.saturating_sub(max_dist);
        let end = (i + max_dist + 1).min(len_b);

        for j in start..end {
            if (hash_b & (1 << j)) == 0 && a_chars[i] == b_chars[j] {
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

    for i in 0..len_a {
        if (hash_a & (1 << i)) != 0 {
            while (hash_b & (1 << b_ptr)) == 0 {
                b_ptr += 1;
            }
            if a_chars[i] != b_chars[b_ptr] {
                transpositions += 1.0;
            }
            b_ptr += 1;
        }
    }

    let jaro_similarity = (1.0 / 3.0) * (
        matches / len_a as f32 +
            matches / len_b as f32 +
            (matches - transpositions / 2.0) / matches
    );

    let mut prefix_len = 0;
    for (c1, c2) in a_chars.into_iter().zip(b_chars) {
        if c1 == c2 {
            prefix_len += 1;
        } else {
            break;
        }
    }

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
}
