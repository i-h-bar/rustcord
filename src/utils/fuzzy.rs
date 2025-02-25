use std::cmp;

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


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_lev() {
        assert_eq!(lev("sitting", "kitten"), 3)
    }

}
