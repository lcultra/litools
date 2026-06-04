use std::sync::Arc;

use crate::{SearchResult, provider::SearchProvider, query::SearchQuery, ranking::rank_results};

#[derive(Default)]
pub struct SearchEngine {
    providers: Vec<Arc<dyn SearchProvider>>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_provider(&mut self, provider: Arc<dyn SearchProvider>) {
        self.providers.push(provider);
    }

    pub fn search(&self, query: SearchQuery) -> Vec<SearchResult> {
        self.search_with_providers(query, std::iter::empty::<&str>())
    }

    pub fn search_with_providers<'a>(
        &self,
        query: SearchQuery,
        enabled_provider_ids: impl IntoIterator<Item = &'a str>,
    ) -> Vec<SearchResult> {
        let enabled_provider_ids = enabled_provider_ids.into_iter().collect::<Vec<_>>();
        let mut results = Vec::new();

        for provider in &self.providers {
            if enabled_provider_ids.is_empty() || enabled_provider_ids.contains(&provider.id()) {
                results.extend(provider.search(&query));
            }
        }

        rank_results(results, query.limit)
    }
}
