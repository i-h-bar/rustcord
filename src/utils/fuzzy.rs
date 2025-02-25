use crate::mtg::db::FuzzyFound;
use std::cmp::Ordering;
use std::fmt::Display;

fn jaro_winkler(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }

    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let len_a = a.len();
    let len_b = b.len();
    let max_dist = (len_a.max(len_b) / 2) - 1;
    let mut matches = 0;
    let mut hash_a: Vec<u8> = vec![0; len_a];
    let mut hash_b: Vec<u8> = vec![0; len_b];

    for i in 0..len_a {
        for j in 0.max(i as isize - max_dist as isize) as usize..len_b.min(i + max_dist + 1) {
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

pub fn best_jaro_match_name_fuzzy_found(target: &str, heap: Vec<FuzzyFound>) -> Option<FuzzyFound> {
    let (_, closest_match) = heap
        .into_iter()
        .map(|card| {
            let distance = jaro_winkler(target, &card.front_normalised_name);
            (distance, card)
        })
        .max_by(|&(x, _), (y, _)| x.partial_cmp(y).unwrap_or_else(|| Ordering::Less))?;

    Some(closest_match)
}

pub fn best_jaro_match(target: &str, heap: Vec<String>) -> Option<String> {
    let (_, closest_match) = heap
        .into_iter()
        .map(|needle| {
            let distance = jaro_winkler(target, &needle);
            (distance, needle)
        })
        .max_by(|&(x, _), (y, _)| x.partial_cmp(y).unwrap_or_else(|| Ordering::Less))?;

    Some(closest_match)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaro_winkler() {
        let a = "CRATE";
        let b = "TRACE";

        assert_eq!(jaro_winkler(a, b), 0.73333335)
    }
}
