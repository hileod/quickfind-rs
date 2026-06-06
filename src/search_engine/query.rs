use super::tokenizer::{QueryTerm, tokenize};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedQuery {
    pub raw: String,
    pub terms: Vec<QueryTerm>,
}

impl ParsedQuery {
    pub fn parse(raw: &str) -> Self {
        Self {
            raw: raw.trim().to_string(),
            terms: tokenize(raw),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty() || self.terms.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_raw_and_terms() {
        let query = ParsedQuery::parse(" report 2026 ");

        assert_eq!(query.raw, "report 2026");
        assert_eq!(query.terms.len(), 2);
    }
}
