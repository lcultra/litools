use litools_search::{SearchEngine, SearchQuery, SearchResult};

use crate::app::LitoolsApp;
use crate::command::BuiltinCommandProvider;

use super::DEFAULT_LAUNCHER_RESULT_LIMIT;

impl LitoolsApp {
    pub fn search(&self, text: impl Into<String>) -> Vec<SearchResult> {
        let settings = self.context.settings.get();
        self.context.search.search_with_providers(
            SearchQuery::with_limit(text, DEFAULT_LAUNCHER_RESULT_LIMIT),
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

/// 创建搜索引擎并注册内置 provider（BuiltinCommandProvider）。
///
/// LauncherPlugin 和 PluginHostPlugin 在 bootstrap 时通过 InternalPlugin 注册。
pub(crate) fn default_search_engine() -> SearchEngine {
    let mut search = SearchEngine::new();
    search.register_provider(std::sync::Arc::new(BuiltinCommandProvider));
    search
}
