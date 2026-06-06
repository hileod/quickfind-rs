pub fn is_subsequence(candidate: &str, query: &str) -> bool {
    let mut chars = candidate.chars();
    query.chars().all(|needle| chars.any(|hay| hay == needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_subsequence_match() {
        assert!(is_subsequence("quickfind", "qf"));
        assert!(!is_subsequence("quickfind", "zq"));
    }
}
