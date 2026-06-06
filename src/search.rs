use crate::index::FileEntry;

#[derive(Debug)]
pub struct SearchHit<'a> {
    pub entry: &'a FileEntry,
    pub score: i64,
}

pub fn search<'a>(entries: &'a [FileEntry], query: &str, limit: usize) -> Vec<SearchHit<'a>> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let terms = query.split_whitespace().collect::<Vec<_>>();

    let mut hits = Vec::new();
    for entry in entries {
        if let Some(score) = score_entry(entry, &query, &terms) {
            hits.push(SearchHit { entry, score });
        }
    }

    hits.sort_unstable_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.entry.path.len().cmp(&right.entry.path.len()))
            .then_with(|| left.entry.lower_path.cmp(&right.entry.lower_path))
    });
    hits.truncate(limit);
    hits
}

fn score_entry(entry: &FileEntry, query: &str, terms: &[&str]) -> Option<i64> {
    if terms.len() > 1 {
        return score_terms(entry, terms);
    }

    let mut best = None;

    if let Some(pos) = entry.lower_name.find(query) {
        let exact_bonus = if entry.lower_name == query { 2_000 } else { 0 };
        best = Some(10_000 + exact_bonus - pos as i64 - entry.name().len() as i64 / 8);
    }

    if let Some(pos) = entry.lower_path.find(query) {
        let score = 6_000 - pos as i64 / 2 - entry.path.len() as i64 / 16;
        best = Some(best.map_or(score, |current: i64| current.max(score)));
    }

    if best.is_none() {
        best = fuzzy_score(&entry.lower_name, query).map(|score| score + 2_000);
    }

    if best.is_none() && query.len() >= 3 {
        best = fuzzy_score(&entry.lower_path, query);
    }

    best
}

fn score_terms(entry: &FileEntry, terms: &[&str]) -> Option<i64> {
    let mut total = 0;
    let mut name_hits = 0;

    for term in terms {
        let score = score_entry(entry, term, &[*term])?;
        if entry.lower_name.contains(term) || fuzzy_score(&entry.lower_name, term).is_some() {
            name_hits += 1;
        }
        total += score;
    }

    Some(total + name_hits * 750 - terms.len() as i64 * entry.path.len() as i64 / 64)
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
    use super::*;

    #[test]
    fn filename_match_beats_path_match() {
        let entries = vec![
            FileEntry::from_string(r"C:\cargo\notes.txt".to_string()),
            FileEntry::from_string(r"C:\work\Cargo.toml".to_string()),
        ];

        let hits = search(&entries, "cargo", 10);

        assert_eq!(hits[0].entry.name(), "Cargo.toml");
    }

    #[test]
    fn fuzzy_finds_subsequence() {
        let entries = vec![FileEntry::from_string(
            r"C:\Users\liam\Documents\quickfind.rs".to_string(),
        )];

        let hits = search(&entries, "qf", 10);

        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn multi_term_query_matches_separated_filename_parts() {
        let entries = vec![
            FileEntry::from_string(r"C:\work\project-final-report.docx".to_string()),
            FileEntry::from_string(r"C:\work\project-notes.txt".to_string()),
        ];

        let hits = search(&entries, "project report", 10);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.name(), "project-final-report.docx");
    }

    #[test]
    fn multi_term_query_can_match_across_path_and_name() {
        let entries = vec![FileEntry::from_string(
            r"C:\Users\liam\Downloads\invoice.pdf".to_string(),
        )];

        let hits = search(&entries, "downloads invoice", 10);

        assert_eq!(hits.len(), 1);
    }
}
