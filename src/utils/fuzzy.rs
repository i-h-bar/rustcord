use std::cmp::Ordering;
use std::str::Chars;

pub trait ToChars {
    fn to_chars(&self) -> Chars<'_>;
}

impl ToChars for str {
    fn to_chars(&self) -> Chars<'_> {
        self.chars()
    }
}

impl ToChars for &str {
    fn to_chars(&self) -> Chars<'_> {
        self.chars()
    }
}

impl ToChars for &String {
    fn to_chars(&self) -> Chars<'_> {
        self.chars()
    }
}

impl ToChars for String {
    fn to_chars(&self) -> Chars<'_> {
        self.chars()
    }
}

impl ToChars for Box<str> {
    fn to_chars(&self) -> Chars<'_> {
        self.chars()
    }
}

#[allow(clippy::cast_precision_loss)]
pub fn jaro_winkler<A: PartialEq<B> + ToChars, B: ToChars>(a: &A, b: &B) -> f32 {
    if a == b {
        return 1.0;
    }

    let a: Vec<char> = a.to_chars().collect();
    let b: Vec<char> = b.to_chars().collect();
    let len_a = a.len();
    let len_b = b.len();
    let max_dist = (len_a.max(len_b) / 2) - 1;
    let mut matches = 0;
    let mut hash_a: Vec<u8> = vec![0; len_a];
    let mut hash_b: Vec<u8> = vec![0; len_b];

    for i in 0..len_a {
        let i_isize = i32::try_from(i).unwrap_or_default();
        let max_isize = i32::try_from(max_dist).unwrap_or_default();
        #[allow(clippy::cast_sign_loss)]
        for j in 0.max(i_isize - max_isize) as usize..len_b.min(i + max_dist + 1) {
            if a[i] == b[j] && hash_b[j] == 0 {
                hash_a[i] = 1;
                hash_b[j] = 1;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut t = 0;
    let mut point = 0;

    for i in 0..len_a {
        if hash_a[i] != 0 {
            while hash_b[point] == 0 {
                point += 1;
            }

            if a[i] != b[point] {
                t += 1;
            }
            point += 1;
        }
    }

    let t = t / 2;

    let match_diff = (matches - t) as f32;
    let matches = matches as f32;
    (matches / len_a as f32 + matches / len_b as f32 + match_diff / matches) / 3.0
}

pub fn winkliest_match<
    A: PartialEq<B> + ToChars,
    B: ToChars,
    I: AsRef<[B]> + IntoIterator<Item = B>,
>(
    target: &A,
    heap: I,
) -> Option<B> {
    let (_, closest_match) = heap
        .into_iter()
        .map(|needle| {
            let distance = jaro_winkler(target, &needle);
            (distance, needle)
        })
        .max_by(|&(x, _), (y, _)| x.partial_cmp(y).unwrap_or(Ordering::Less))?;

    Some(closest_match)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaro_winkler() {
        let a = "CRATE";
        let b = "TRACE";

        assert_eq!(jaro_winkler(&a, &b), 0.73333335)
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
