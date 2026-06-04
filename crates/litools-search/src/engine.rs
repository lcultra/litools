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
        let mut results = Vec::new();

        for provider in &self.providers {
            results.extend(provider.search(&query));
        }

        rank_results(results, query.limit)
    }
}
