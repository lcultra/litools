use crate::provider::SearchResult;

pub fn rank_results(mut results: Vec<SearchResult>, limit: Option<usize>) -> Vec<SearchResult> {
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
