#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QueryTerm {
    pub text: String,
}

pub fn tokenize(query: &str) -> Vec<QueryTerm> {
    query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .map(|term| QueryTerm {
            text: term.to_lowercase(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_query_terms() {
        assert_eq!(
            tokenize("Project Report")
                .into_iter()
                .map(|term| term.text)
                .collect::<Vec<_>>(),
            vec!["project", "report"]
        );
    }
}
