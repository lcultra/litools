use std::sync::Arc;

use litools_index::IndexDatabase;
use litools_search::{SearchEngine, SearchQuery, SearchResult};

use crate::{
    app::LitoolsApp,
    app_provider::AppSearchProvider,
    command::BuiltinCommandProvider,
    plugin_provider::PluginCommandProvider,
};

use super::LAUNCHER_RESULT_LIMIT;

impl LitoolsApp {
    pub fn search(&self, text: impl Into<String>) -> Vec<SearchResult> {
        let settings = self.context.settings.get();
        self.context.search.search_with_providers(
            SearchQuery::with_limit(text, LAUNCHER_RESULT_LIMIT),
            settings.search.enabled_providers.iter().map(String::as_str),
        )
    }

    pub(crate) fn search_without_limit(&self, text: impl Into<String>) -> Vec<SearchResult> {
        let settings = self.context.settings.get();
        self.context.search.search_with_providers(
            SearchQuery::without_limit(text),
            settings.search.enabled_providers.iter().map(String::as_str),
        )
    }
}

pub(crate) fn default_search_engine(
    database: IndexDatabase,
) -> (SearchEngine, Arc<PluginCommandProvider>) {
    let app_provider = Arc::new(AppSearchProvider::new(database.clone()));
    let plugin_provider = Arc::new(PluginCommandProvider::new(database));
    let mut search = SearchEngine::new();
    search.register_provider(Arc::new(BuiltinCommandProvider));
    search.register_provider(app_provider);
    search.register_provider(plugin_provider.clone());
    (search, plugin_provider)
}
