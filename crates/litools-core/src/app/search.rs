use std::sync::Arc;

use litools_index::IndexDatabase;
use litools_plugin::PluginManager;
use litools_search::{SearchEngine, SearchQuery, SearchResult};

use crate::{
    app::LitoolsApp, command::BuiltinCommandProvider, plugin_provider::PluginCommandProvider,
};

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

/// 创建搜索引擎，内置注册 BuiltinCommandProvider 和 PluginCommandProvider。
///
/// AppSearchProvider 不再在此硬编码——由 LauncherPlugin 在 bootstrap 时通过
/// InternalPlugin::search_providers() 注册。
pub(crate) fn default_search_engine(
    _database: IndexDatabase,
    plugin_manager: Arc<PluginManager>,
) -> (SearchEngine, Arc<PluginCommandProvider>) {
    let plugin_provider = Arc::new(PluginCommandProvider::new());
    plugin_provider.set_plugin_manager(plugin_manager);
    let mut search = SearchEngine::new();
    search.register_provider(Arc::new(BuiltinCommandProvider));
    search.register_provider(plugin_provider.clone());
    (search, plugin_provider)
}
