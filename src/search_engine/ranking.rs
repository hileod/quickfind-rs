use super::document::SearchDocument;
use super::query::ParsedQuery;

pub fn score_document(document: &SearchDocument<'_>, query: &ParsedQuery) -> Option<i64> {
    if query.terms.len() > 1 {
        return score_terms(document, query);
    }

    score_single(document, &query.raw)
}

pub fn filter_only_score(document: &SearchDocument<'_>) -> i64 {
    let folder_bonus = if document.is_folder() { 180 } else { 0 };
    1_000 + folder_bonus - document.entry.path.len() as i64 / 32
}

fn score_terms(document: &SearchDocument<'_>, query: &ParsedQuery) -> Option<i64> {
    let mut total = 0;
    let mut name_hits = 0;

    for term in &query.terms {
        let score = score_single(document, &term.text)?;
        if document.lower_name.contains(&term.text)
            || fuzzy_score(document.lower_name, &term.text).is_some()
        {
            name_hits += 1;
        }
        total += score;
    }

    Some(total + name_hits * 750 - query.terms.len() as i64 * document.entry.path.len() as i64 / 64)
}

fn score_single(document: &SearchDocument<'_>, query: &str) -> Option<i64> {
    let mut best = None;

    if document.lower_name == query {
        best = Some(14_000 - document.name.len() as i64 / 8);
    } else if document.lower_name.starts_with(query) {
        best = Some(12_000 - document.name.len() as i64 / 8);
    } else if let Some(pos) = document.lower_name.find(query) {
        best = Some(10_000 - pos as i64 - document.name.len() as i64 / 8);
    }

    if let Some(pos) = document.lower_path.find(query) {
        let score = 6_000 - pos as i64 / 2 - document.entry.path.len() as i64 / 16;
        best = Some(best.map_or(score, |current: i64| current.max(score)));
    }

    if best.is_none() {
        best = fuzzy_score(document.lower_name, query).map(|score| score + 2_000);
    }

    if best.is_none() && query.len() >= 3 {
        best = fuzzy_score(document.lower_path, query);
    }

    best.map(|score| {
        let folder_bonus = if document.is_folder() { 180 } else { 0 };
        let extension_bonus = if document.extension.as_deref() == Some(query) {
            120
        } else {
            0
        };
        score + folder_bonus + extension_bonus
    })
}

fn fuzzy_score(candidate: &str, query: &str) -> Option<i64> {
    let mut score = 0;
    let mut last_match = None;
    let mut search_from = 0;

    for ch in query.chars() {
        let haystack = &candidate[search_from..];
        let offset = haystack.find(ch)?;
        let index = search_from + offset;
        score += if Some(index.saturating_sub(1)) == last_match {
            90
        } else {
            35
        };
        if index == 0
            || matches!(
                candidate.as_bytes().get(index - 1),
                Some(b'\\' | b'/' | b'-' | b'_' | b'.' | b' ')
            )
        {
            score += 45;
        }
        last_match = Some(index);
        search_from = index + ch.len_utf8();
    }

    Some(score - candidate.len() as i64 / 8)
}

#[cfg(test)]
mod tests {
    use crate::index::{EntryKind, FileEntry};

    use super::*;

    #[test]
    fn exact_name_beats_contains() {
        let exact = FileEntry::from_string(r"C:\work\cargo".to_string());
        let contains = FileEntry::from_string(r"C:\work\my-cargo-notes.txt".to_string());
        let query = ParsedQuery::parse("cargo");

        let exact_score = score_document(&SearchDocument::from_entry(&exact), &query).unwrap();
        let contains_score =
            score_document(&SearchDocument::from_entry(&contains), &query).unwrap();

        assert!(exact_score > contains_score);
    }

    #[test]
    fn folder_gets_small_bonus() {
        let folder =
            FileEntry::from_string_with_kind(r"C:\work\cargo".to_string(), EntryKind::Directory);
        let file = FileEntry::from_string(r"C:\work\cargo".to_string());
        let query = ParsedQuery::parse("cargo");

        let folder_score = score_document(&SearchDocument::from_entry(&folder), &query).unwrap();
        let file_score = score_document(&SearchDocument::from_entry(&file), &query).unwrap();

        assert!(folder_score > file_score);
    }
}
