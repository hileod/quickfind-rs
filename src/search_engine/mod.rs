pub mod document;
pub mod fuzzy_match;
pub mod query;
pub mod ranking;
pub mod tokenizer;

use std::cmp::Ordering;

use crate::index::FileEntry;

use document::SearchDocument;
use query::ParsedQuery;

#[derive(Debug)]
pub struct SearchHit<'a> {
    pub entry: &'a FileEntry,
    pub score: i64,
}

#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub kind: Option<crate::index::EntryKind>,
    pub extension: Option<String>,
    pub drive: Option<String>,
}

pub fn search<'a>(entries: &'a [FileEntry], query: &str, limit: usize) -> Vec<SearchHit<'a>> {
    search_with_filters(entries, query, limit, &SearchFilters::default())
}

pub fn search_with_filters<'a>(
    entries: &'a [FileEntry],
    query: &str,
    limit: usize,
    filters: &SearchFilters,
) -> Vec<SearchHit<'a>> {
    let (query_text, wildcard_extension) = extract_wildcard_extension(query);
    let Some(filters) = merge_wildcard_filter(filters, wildcard_extension) else {
        return Vec::new();
    };
    let query = ParsedQuery::parse(&query_text);
    let filter_only = query.is_empty() && filters.has_any();

    if query.is_empty() && !filter_only {
        return Vec::new();
    }

    let mut hits = Vec::new();
    for entry in entries {
        let document = SearchDocument::from_entry(entry);
        if !document.matches_filters(&filters) {
            continue;
        }
        if filter_only {
            hits.push(SearchHit {
                entry,
                score: ranking::filter_only_score(&document),
            });
        } else if let Some(score) = ranking::score_document(&document, &query) {
            hits.push(SearchHit { entry, score });
        }
    }

    hits.sort_unstable_by(compare_hits);
    hits.truncate(limit);
    hits
}

impl SearchFilters {
    fn has_any(&self) -> bool {
        self.kind.is_some() || self.extension.is_some() || self.drive.is_some()
    }
}

fn extract_wildcard_extension(query: &str) -> (String, Option<String>) {
    let mut wildcard_extension = None;
    let mut terms = Vec::new();

    for term in query.split_whitespace() {
        if let Some(extension) = term.strip_prefix("*.") {
            let extension = extension.trim().to_ascii_lowercase();
            if !extension.is_empty() {
                wildcard_extension = Some(extension);
            }
        } else {
            terms.push(term);
        }
    }

    (terms.join(" "), wildcard_extension)
}

fn merge_wildcard_filter(
    filters: &SearchFilters,
    wildcard_extension: Option<String>,
) -> Option<SearchFilters> {
    let Some(wildcard_extension) = wildcard_extension else {
        return Some(filters.clone());
    };

    if let Some(existing) = filters.extension.as_deref()
        && existing != wildcard_extension
    {
        return None;
    }

    let mut merged = filters.clone();
    merged.extension = Some(wildcard_extension);
    Some(merged)
}

fn compare_hits(left: &SearchHit<'_>, right: &SearchHit<'_>) -> Ordering {
    right
        .score
        .cmp(&left.score)
        .then_with(|| left.entry.path.len().cmp(&right.entry.path.len()))
        .then_with(|| left.entry.lower_path.cmp(&right.entry.lower_path))
}

#[cfg(test)]
mod tests {
    use crate::index::{EntryKind, FileEntry};

    use super::*;

    #[test]
    fn searches_multi_term_filename() {
        let entries = vec![
            FileEntry::from_string(r"C:\work\project-final-report.docx".to_string()),
            FileEntry::from_string(r"C:\work\project-notes.txt".to_string()),
        ];

        let hits = search(&entries, "project report", 10);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.name(), "project-final-report.docx");
    }

    #[test]
    fn filters_by_kind() {
        let entries = vec![
            FileEntry::from_string_with_kind(r"C:\work\reports".to_string(), EntryKind::Directory),
            FileEntry::from_string(r"C:\work\reports.txt".to_string()),
        ];
        let filters = SearchFilters {
            kind: Some(EntryKind::Directory),
            ..SearchFilters::default()
        };

        let hits = search_with_filters(&entries, "reports", 10, &filters);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.kind, EntryKind::Directory);
    }

    #[test]
    fn filters_by_extension() {
        let entries = vec![
            FileEntry::from_string(r"C:\work\report.pdf".to_string()),
            FileEntry::from_string(r"C:\work\report.docx".to_string()),
        ];
        let filters = SearchFilters {
            extension: Some("pdf".to_string()),
            ..SearchFilters::default()
        };

        let hits = search_with_filters(&entries, "report", 10, &filters);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.name(), "report.pdf");
    }

    #[test]
    fn wildcard_extension_query_lists_matching_files() {
        let entries = vec![
            FileEntry::from_string(r"C:\work\a.pdf".to_string()),
            FileEntry::from_string(r"C:\work\b.docx".to_string()),
        ];

        let hits = search(&entries, "*.pdf", 10);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.name(), "a.pdf");
    }

    #[test]
    fn wildcard_extension_can_combine_with_query_terms() {
        let entries = vec![
            FileEntry::from_string(r"C:\work\budget-report.pdf".to_string()),
            FileEntry::from_string(r"C:\work\budget-report.xlsx".to_string()),
        ];

        let hits = search(&entries, "budget *.pdf", 10);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.name(), "budget-report.pdf");
    }
}
