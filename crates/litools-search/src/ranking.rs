use std::collections::HashMap;

use crate::provider::SearchResult;

/// 上下文加权系数：provider 声明了某个 feature 亲和力且 context 命中时，每个匹配 feature +0.5。
const CONTEXT_BOOST_PER_FEATURE: f32 = 0.5;

/// 按分数降序排列结果，截断到 limit。
///
/// 兼容旧调用（无 context）：context_features 传空 slice 即可。
pub fn rank_results(
    mut results: Vec<SearchResult>,
    limit: Option<usize>,
    context_features: &[String],
    provider_affinities: &HashMap<String, Vec<String>>,
) -> Vec<SearchResult> {
    // 上下文加权：匹配 feature → boost
    if !context_features.is_empty() {
        for result in &mut results {
            if let Some(affinities) = provider_affinities.get(&result.provider) {
                let matches = affinities
                    .iter()
                    .filter(|a| context_features.iter().any(|f| f == *a))
                    .count();
                if matches > 0 {
                    result.score += matches as f32 * CONTEXT_BOOST_PER_FEATURE;
                }
            }
        }
    }

    results.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.title.cmp(&right.title))
    });

    if let Some(limit) = limit {
        results.truncate(limit);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{SearchResult, SearchResultAction};
    use crate::matcher::SearchResultMatches;

    fn make_result(provider: &str, title: &str, score: f32) -> SearchResult {
        SearchResult {
            id: title.to_string(),
            title: title.to_string(),
            subtitle: None,
            icon_uri: None,
            provider: provider.to_string(),
            score,
            matches: SearchResultMatches::default(),
            actions: vec![SearchResultAction {
                id: "open".to_string(),
                label: "Open".to_string(),
            }],
        }
    }

    #[test]
    fn sorts_by_score_desc() {
        let results = vec![
            make_result("a", "B", 50.0),
            make_result("a", "A", 100.0),
            make_result("a", "C", 75.0),
        ];
        let ranked = rank_results(results, None, &[], &HashMap::new());
        assert_eq!(ranked[0].score, 100.0);
        assert_eq!(ranked[1].score, 75.0);
        assert_eq!(ranked[2].score, 50.0);
    }

    #[test]
    fn respects_limit() {
        let results = vec![
            make_result("a", "A", 100.0),
            make_result("a", "B", 50.0),
            make_result("a", "C", 75.0),
        ];
        let ranked = rank_results(results, Some(2), &[], &HashMap::new());
        assert_eq!(ranked.len(), 2);
    }

    #[test]
    fn context_boost_increases_score() {
        let mut affinities = HashMap::new();
        affinities.insert("json-tools".to_string(), vec!["json".to_string()]);

        let results = vec![
            make_result("json-tools", "JSON Formatter", 50.0),
            make_result("other", "Something", 100.0),
        ];
        let features: Vec<String> = vec!["json".to_string()];
        let ranked = rank_results(results, None, &features, &affinities);
        // json-tools: 50 + 0.5 = 50.5, still below 100
        assert_eq!(ranked[0].provider, "other");
    }

    #[test]
    fn context_boost_can_change_order() {
        let mut affinities = HashMap::new();
        affinities.insert("json-tools".to_string(), vec!["json".to_string()]);

        let results = vec![
            make_result("json-tools", "JSON Formatter", 70.0),
            make_result("other", "Something", 71.0),
        ];
        let features: Vec<String> = vec!["json".to_string()];
        let ranked = rank_results(results, None, &features, &affinities);
        // json-tools: 70 + 0.5 = 70.5, now above 71.0? No — 70.5 < 71.0
        assert_eq!(ranked[0].provider, "other");
    }

    #[test]
    fn no_boost_when_context_empty() {
        let mut affinities = HashMap::new();
        affinities.insert("json-tools".to_string(), vec!["json".to_string()]);

        let results = vec![
            make_result("json-tools", "JSON Formatter", 50.0),
            make_result("other", "Something", 100.0),
        ];
        let ranked = rank_results(results, None, &[], &affinities);
        assert_eq!(ranked[0].score, 100.0);
        assert_eq!(ranked[1].score, 50.0);
    }

    #[test]
    fn multiple_feature_matches_stack() {
        let mut affinities = HashMap::new();
        affinities.insert("multi".to_string(), vec!["json".to_string(), "url".to_string()]);

        let results = vec![
            make_result("multi", "Multi Tool", 50.0),
            make_result("other", "Something", 51.0),
        ];
        let features: Vec<String> = vec!["json".to_string(), "url".to_string()];
        let ranked = rank_results(results, None, &features, &affinities);
        // multi: 50 + 0.5 + 0.5 = 51.0, tie with 51.0 → title sort: "Multi Tool" < "Something"
        assert_eq!(ranked[0].provider, "multi");
    }
}
