//! Small string-distance utilities shared by vocabulary suggestion paths.

/// Returns the Levenshtein edit distance between the characters of `a` and `b`.
///
/// This deliberately narrow API uses two rolling rows. Local names are short, so
/// the quadratic cost is trivial; allocation is limited to those rows and the
/// character buffers.
/// [SONNET-4.6] sq-lxw27, gh-3694
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::edit_distance;

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("Task", "Tas"), 1);
        assert_eq!(edit_distance("Person", "Persons"), 1);
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("é", "e"), 1);
    }
}
