use litools_search::{SearchQuery, SearchResult};

use crate::app::LitoolsApp;

use super::DEFAULT_LAUNCHER_RESULT_LIMIT;

impl LitoolsApp {
    pub async fn search(&self, text: impl Into<String>) -> Vec<SearchResult> {
        let settings = self.context.settings.get();
        self.context
            .search
            .search_with_providers(
                SearchQuery::with_limit(text, DEFAULT_LAUNCHER_RESULT_LIMIT),
                settings.search.enabled_providers.iter().map(String::as_str),
            )
            .await
    }

}
