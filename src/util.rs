//! Small shared utilities.

/// Levenshtein edit distance between two strings (insertions, deletions,
/// substitutions). Used for "did you mean?" suggestions.
#[must_use]
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Returns the candidate closest to `target` within `max_distance`, if any.
#[must_use]
pub fn closest<'a>(target: &str, candidates: &[&'a str], max_distance: usize) -> Option<&'a str> {
    candidates
        .iter()
        .map(|c| (levenshtein(target, c), *c))
        .filter(|(d, _)| *d <= max_distance)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_basics() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("logc", "logic"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn closest_within_threshold() {
        let candidates = ["api", "default", "logic", "perf"];
        assert_eq!(closest("logc", &candidates, 2), Some("logic"));
        assert_eq!(closest("defalt", &candidates, 2), Some("default"));
        // Too far -> no suggestion.
        assert_eq!(closest("zzzzzz", &candidates, 2), None);
    }
}
