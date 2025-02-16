use rayon::prelude::*;
use std::cmp;
use std::sync::Arc;

pub fn lev(a: &str, b: &str) -> usize {
    if a.is_empty() {
        return b.chars().count();
    } else if b.is_empty() {
        return a.chars().count();
    }

    let mut dcol: Vec<usize> = (0..(b.len() + 1)).collect();
    let mut t_last = 0;
    for (i, sc) in a.chars().enumerate() {
        let mut current = i;
        dcol[0] = current + 1;
        for (j, tc) in b.chars().enumerate() {
            let next = dcol[j + 1];
            if sc == tc {
                dcol[j + 1] = current;
            } else {
                dcol[j + 1] = cmp::min(current, next);
                dcol[j + 1] = cmp::min(dcol[j + 1], dcol[j]) + 1;
            }
            current = next;
            t_last = j;
        }
    }
    dcol[t_last + 1]
}

pub fn best_match_lev<'a, I: IntoParallelRefIterator<'a, Item = &'a Arc<str>>>(
    a: &str,
    items: &'a I,
) -> Option<&'a Arc<str>> {
    Some(
        items
            .par_iter()
            .map(|item: &Arc<str>| {
                let dist = lev(&a, item.as_ref());
                (item, dist)
            })
            .min_by(|(_, x), (_, y)| x.cmp(y))?
            .0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_lev() {
        assert_eq!(lev("sitting", "kitten"), 3)
    }

    #[test]
    fn test_best_match() {
        let a = vec!["sitting".to_string(), "kitten".to_string()];
        let b = "sitting";

        assert_eq!(best_match_lev(&b, &a).unwrap(), a.get(0).unwrap());
    }

    #[test]
    fn test_best_match_2() {
        let a = vec!["sitting".to_string(), "kitten".to_string()];
        let b = "setting";

        assert_eq!(best_match_lev(&b, &a).unwrap(), a.get(0).unwrap());
    }

    #[test]
    fn test_best_match_hash_map() {
        let mut a: HashSet<String> = HashSet::new();
        a.insert("sitting".to_string());
        a.insert("kitten".to_string());
        let b = "setting";

        assert_eq!(best_match_lev(&b, &a).unwrap(), &"sitting".to_string());
    }
}
