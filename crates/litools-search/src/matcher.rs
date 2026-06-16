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

// ── FieldMatcher：统一的多字段匹配器 ──
//
// 替代各 SearchProvider 中重复的 "遍历候选字段 → 保留最佳分数" 模式。
// 各 provider 只需声明 FieldWeights 和字段列表，匹配逻辑由 FieldMatcher 统一处理。

/// 字段匹配的分数权重。
///
/// 各 MatchKind 对应的基础分数由 provider 自行配置，
/// 最终分数 = base + adjustment（.max(1.0)）。
#[derive(Clone, Debug)]
pub struct FieldWeights {
    pub exact: f32,
    pub prefix: f32,
    pub contains: f32,
    pub fuzzy_cap: f32,
}

/// 字段对用户是否可见，决定匹配范围是否写入结果。
#[derive(Clone, Copy)]
pub enum VisibleField {
    Title,
    Subtitle,
    Hidden,
}

/// 多字段匹配器：依次考虑各字段的 [`TextMatch`]，保留最佳结果。
///
/// # 使用方式
///
/// ```ignore
/// let weights = FieldWeights { exact: 112.0, prefix: 100.0, contains: 72.0, fuzzy_cap: 68.0 };
/// let mut matcher = FieldMatcher::new();
/// matcher.consider(match_text(title, query), 0.0, VisibleField::Title, &weights);
/// matcher.consider(match_text(subtitle, query), -8.0, VisibleField::Subtitle, &weights);
/// let (score, title_ranges, subtitle_ranges) = matcher.finish();
/// ```
#[derive(Default)]
pub struct FieldMatcher {
    score: f32,
    title_ranges: Vec<MatchRange>,
    subtitle_ranges: Vec<MatchRange>,
}

impl FieldMatcher {
    /// 创建一个初始分数为 0 的匹配器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建一个指定初始分数的匹配器，用于空查询场景。
    pub fn with_score(score: f32) -> Self {
        Self {
            score,
            ..Self::default()
        }
    }

    /// 考虑一个字段的匹配结果。若优于当前最佳匹配则替换。
    pub fn consider(
        &mut self,
        text_match: Option<TextMatch>,
        adjustment: f32,
        visible_field: VisibleField,
        weights: &FieldWeights,
    ) {
        let Some(text_match) = text_match else {
            return;
        };
        let base = match text_match.kind {
            MatchKind::Exact => weights.exact,
            MatchKind::Prefix => weights.prefix,
            MatchKind::Contains => weights.contains,
            MatchKind::Fuzzy => text_match.score.min(weights.fuzzy_cap),
        };
        let score = (base + adjustment).max(1.0);
        if self.score >= score {
            return;
        }
        self.score = score;
        self.title_ranges = match visible_field {
            VisibleField::Title => text_match.ranges.clone(),
            _ => Vec::new(),
        };
        self.subtitle_ranges = match visible_field {
            VisibleField::Subtitle => text_match.ranges,
            _ => Vec::new(),
        };
    }

    /// 取出最终结果：`(score, title_ranges, subtitle_ranges)`。
    pub fn finish(self) -> (f32, Vec<MatchRange>, Vec<MatchRange>) {
        (self.score, self.title_ranges, self.subtitle_ranges)
    }

    /// 是否有任何匹配（分数 > 0）。
    pub fn has_match(&self) -> bool {
        self.score > 0.0
    }
}

#[cfg(test)]
mod field_matcher_tests {
    use super::*;

    const TEST_WEIGHTS: FieldWeights = FieldWeights {
        exact: 100.0,
        prefix: 80.0,
        contains: 60.0,
        fuzzy_cap: 50.0,
    };

    #[test]
    fn picks_best_score_across_fields() {
        // Title: contains match → 60.0; Subtitle: exact match → 100.0
        let mut matcher = FieldMatcher::new();
        matcher.consider(
            match_text("Hello World", "wor"),
            0.0,
            VisibleField::Title,
            &TEST_WEIGHTS,
        );
        matcher.consider(
            match_text("Text", "text"),
            0.0,
            VisibleField::Subtitle,
            &TEST_WEIGHTS,
        );
        let (score, title, subtitle) = matcher.finish();
        // exact match "text" (100.0) beats contains match "wor" (60.0)
        assert!(score > 90.0);
        assert!(title.is_empty());
        assert!(!subtitle.is_empty());
    }

    #[test]
    fn hidden_field_does_not_produce_ranges() {
        let mut matcher = FieldMatcher::new();
        matcher.consider(
            match_text("keyword", "key"),
            0.0,
            VisibleField::Hidden,
            &TEST_WEIGHTS,
        );
        let (score, title, subtitle) = matcher.finish();
        assert!(score > 0.0);
        assert!(title.is_empty());
        assert!(subtitle.is_empty());
    }

    #[test]
    fn adjustment_penalizes_score() {
        let mut matcher = FieldMatcher::new();
        matcher.consider(
            match_text("Hello", "hello"),
            -20.0,
            VisibleField::Title,
            &TEST_WEIGHTS,
        );
        let (score, _, _) = matcher.finish();
        // exact=100.0 - 20.0 = 80.0
        assert_eq!(score, 80.0);
    }

    #[test]
    fn empty_matcher_has_no_match() {
        let matcher = FieldMatcher::new();
        assert!(!matcher.has_match());
    }

    #[test]
    fn with_score_initializes() {
        let matcher = FieldMatcher::with_score(95.0);
        assert!(matcher.has_match());
        let (score, _, _) = matcher.finish();
        assert_eq!(score, 95.0);
    }
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
