use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultMatches {
    #[serde(default)]
    pub title: Vec<MatchRange>,
    #[serde(default)]
    pub subtitle: Vec<MatchRange>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchKind {
    Exact,
    Prefix,
    Contains,
    Fuzzy,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextMatch {
    pub score: f32,
    pub ranges: Vec<MatchRange>,
    pub kind: MatchKind,
}

pub fn normalize_query(query: &str) -> String {
    query.trim().to_lowercase()
}

pub fn match_text(candidate: &str, query: &str) -> Option<TextMatch> {
    let query = normalize_query(query);
    if query.is_empty() || candidate.trim().is_empty() {
        return None;
    }

    let candidate_lower = candidate.to_lowercase();
    let candidate_char_count = candidate.chars().count();
    let query_char_count = query.chars().count();

    if candidate_lower == query {
        return Some(TextMatch {
            score: 100.0,
            ranges: vec![MatchRange {
                start: 0,
                end: candidate_char_count,
            }],
            kind: MatchKind::Exact,
        });
    }

    if candidate_lower.starts_with(&query) {
        return Some(TextMatch {
            score: 90.0,
            ranges: vec![MatchRange {
                start: 0,
                end: query_char_count,
            }],
            kind: MatchKind::Prefix,
        });
    }

    if let Some(start_byte) = candidate_lower.find(&query) {
        let start = candidate_lower[..start_byte].chars().count();
        return Some(TextMatch {
            score: 75.0,
            ranges: vec![MatchRange {
                start,
                end: start + query_char_count,
            }],
            kind: MatchKind::Contains,
        });
    }

    fuzzy_match(candidate, &query)
}

fn fuzzy_match(candidate: &str, query: &str) -> Option<TextMatch> {
    let query_chars = query.chars().collect::<Vec<_>>();
    if query_chars.len() < 2 {
        return None;
    }

    let candidate_chars = candidate
        .chars()
        .enumerate()
        .map(|(index, ch)| (index, ch.to_lowercase().collect::<String>()))
        .collect::<Vec<_>>();
    let mut positions = Vec::with_capacity(query_chars.len());
    let mut search_from = 0;

    for query_char in &query_chars {
        let position = candidate_chars[search_from..]
            .iter()
            .position(|(_, candidate_char)| candidate_char.starts_with(*query_char))?;
        let absolute_position = search_from + position;
        positions.push(candidate_chars[absolute_position].0);
        search_from = absolute_position + 1;
    }

    let first = *positions.first()?;
    let last = *positions.last()?;
    let span = last - first + 1;
    if query_chars.len() == 2 && span > 4 {
        return None;
    }

    let gap_count = span.saturating_sub(query_chars.len());
    let start_penalty = first.min(12) as f32 * 1.2;
    let gap_penalty = gap_count.min(12) as f32 * 2.0;
    let length_penalty = candidate_chars
        .len()
        .saturating_sub(query_chars.len())
        .min(20) as f32
        * 0.25;
    let score = (70.0 - start_penalty - gap_penalty - length_penalty).max(40.0);

    Some(TextMatch {
        score,
        ranges: merge_ranges(
            positions
                .into_iter()
                .map(|position| MatchRange {
                    start: position,
                    end: position + 1,
                })
                .collect(),
        ),
        kind: MatchKind::Fuzzy,
    })
}

pub fn merge_ranges(mut ranges: Vec<MatchRange>) -> Vec<MatchRange> {
    ranges.retain(|range| range.start < range.end);
    ranges.sort_by_key(|range| (range.start, range.end));

    let mut merged: Vec<MatchRange> = Vec::new();
    for range in ranges {
        if let Some(last) = merged.last_mut()
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
            continue;
        }

        merged.push(range);
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_exact_text() {
        let result = match_text("Safari", "safari").expect("exact match");

        assert_eq!(result.kind, MatchKind::Exact);
        assert_eq!(result.ranges, [MatchRange { start: 0, end: 6 }]);
    }

    #[test]
    fn matches_prefix_text() {
        let result = match_text("Safari", "saf").expect("prefix match");

        assert_eq!(result.kind, MatchKind::Prefix);
        assert_eq!(result.ranges, [MatchRange { start: 0, end: 3 }]);
    }

    #[test]
    fn matches_contained_text() {
        let result = match_text("Activity Monitor", "monitor").expect("contains match");

        assert_eq!(result.kind, MatchKind::Contains);
        assert_eq!(result.ranges, [MatchRange { start: 9, end: 16 }]);
    }

    #[test]
    fn matches_fuzzy_text() {
        let result = match_text("Safari", "sfi").expect("fuzzy match");

        assert_eq!(result.kind, MatchKind::Fuzzy);
        assert_eq!(
            result.ranges,
            [
                MatchRange { start: 0, end: 1 },
                MatchRange { start: 2, end: 3 },
                MatchRange { start: 5, end: 6 }
            ]
        );
    }

    #[test]
    fn rejects_out_of_order_fuzzy_text() {
        assert!(match_text("Safari", "ifs").is_none());
    }

    #[test]
    fn matches_chinese_character_ranges() {
        let result = match_text("微信", "微").expect("contains match");

        assert_eq!(result.ranges, [MatchRange { start: 0, end: 1 }]);
    }

    #[test]
    fn merges_adjacent_ranges() {
        assert_eq!(
            merge_ranges(vec![
                MatchRange { start: 0, end: 1 },
                MatchRange { start: 1, end: 2 },
                MatchRange { start: 4, end: 5 },
            ]),
            [
                MatchRange { start: 0, end: 2 },
                MatchRange { start: 4, end: 5 }
            ]
        );
    }

    #[test]
    fn rejects_single_character_fuzzy_match() {
        assert!(match_text("Safari", "f").is_some());
        assert!(match_text("Safari", "r").is_some());
        assert!(match_text("Safari", "x").is_none());
    }
}
