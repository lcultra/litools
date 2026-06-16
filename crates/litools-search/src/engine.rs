use std::collections::HashMap;
use std::sync::Arc;

use crate::{SearchResult, provider::SearchProvider, query::SearchQuery, ranking::rank_results};

#[derive(Default)]
pub struct SearchEngine {
    providers: Vec<Arc<dyn SearchProvider>>,
    plugin_providers: HashMap<String, Vec<String>>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个无主搜索提供者（内部归入 `__builtin__` 分组，统一走插件生命周期路径）。
    pub fn register_provider(&mut self, provider: Arc<dyn SearchProvider>) {
        self.register_plugin_provider("__builtin__", provider);
    }

    /// 注册一个属于指定插件的搜索提供者，以便后续可以通过插件 ID 批量注销。
    pub fn register_plugin_provider(
        &mut self,
        owner_plugin_id: &str,
        provider: Arc<dyn SearchProvider>,
    ) {
        self.plugin_providers
            .entry(owner_plugin_id.to_string())
            .or_default()
            .push(provider.id().to_string());
        self.providers.push(provider);
    }

    /// 注销指定插件拥有的所有搜索提供者。
    pub fn unregister_plugin(&mut self, plugin_id: &str) {
        if let Some(provider_ids) = self.plugin_providers.remove(plugin_id) {
            self.providers
                .retain(|p| !provider_ids.contains(&p.id().to_string()));
        }
    }

    pub fn search(&self, query: SearchQuery) -> Vec<SearchResult> {
        self.search_with_providers(query, std::iter::empty::<&str>())
    }

    #[allow(clippy::needless_lifetimes)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::SearchResultAction;

    struct StubProvider {
        id: &'static str,
        results: Vec<SearchResult>,
    }

    impl SearchProvider for StubProvider {
        fn id(&self) -> &'static str {
            self.id
        }

        fn search(&self, _query: &SearchQuery) -> Vec<SearchResult> {
            self.results.clone()
        }
    }

    fn result_for(title: &str, provider: &str, score: f32) -> SearchResult {
        SearchResult {
            id: title.to_string(),
            title: title.to_string(),
            subtitle: None,
            icon_uri: None,
            provider: provider.to_string(),
            score,
            matches: Default::default(),
            actions: vec![SearchResultAction {
                id: "open".to_string(),
                label: "Open".to_string(),
            }],
        }
    }

    #[test]
    fn empty_engine_returns_empty_results() {
        let engine = SearchEngine::new();
        let results = engine.search(SearchQuery::with_limit("test", 10));
        assert!(results.is_empty());
    }

    #[test]
    fn single_provider_searches() {
        let mut engine = SearchEngine::new();
        engine.register_provider(Arc::new(StubProvider {
            id: "test",
            results: vec![result_for("Alpha", "test", 100.0)],
        }));

        let results = engine.search(SearchQuery::without_limit("alpha"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Alpha");
    }

    #[test]
    fn multiple_providers_are_merged() {
        let mut engine = SearchEngine::new();
        engine.register_provider(Arc::new(StubProvider {
            id: "a",
            results: vec![result_for("A1", "a", 100.0)],
        }));
        engine.register_provider(Arc::new(StubProvider {
            id: "b",
            results: vec![result_for("B1", "b", 90.0)],
        }));

        let results = engine.search(SearchQuery::without_limit(""));
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].provider, "a"); // higher score first
        assert_eq!(results[1].provider, "b");
    }

    #[test]
    fn enabled_provider_ids_filter() {
        let mut engine = SearchEngine::new();
        engine.register_provider(Arc::new(StubProvider {
            id: "a",
            results: vec![result_for("A1", "a", 100.0)],
        }));
        engine.register_provider(Arc::new(StubProvider {
            id: "b",
            results: vec![result_for("B1", "b", 90.0)],
        }));

        let results =
            engine.search_with_providers(SearchQuery::without_limit(""), ["a"].iter().copied());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].provider, "a");
    }

    #[test]
    fn search_respects_limit() {
        let mut engine = SearchEngine::new();
        let mut r = Vec::new();
        for i in 0..5 {
            r.push(result_for(
                &format!("Item{i}"),
                "test",
                (100 - i * 10) as f32,
            ));
        }
        engine.register_provider(Arc::new(StubProvider {
            id: "test",
            results: r,
        }));

        let results = engine.search(SearchQuery::with_limit("", 3));
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn register_plugin_provider_and_unregister() {
        let mut engine = SearchEngine::new();
        engine.register_plugin_provider(
            "plugin_a",
            Arc::new(StubProvider {
                id: "pa",
                results: vec![result_for("Pa1", "pa", 100.0)],
            }),
        );
        engine.register_plugin_provider(
            "plugin_a",
            Arc::new(StubProvider {
                id: "pa2",
                results: vec![result_for("Pa2", "pa2", 80.0)],
            }),
        );
        engine.register_plugin_provider(
            "plugin_b",
            Arc::new(StubProvider {
                id: "pb",
                results: vec![result_for("Pb1", "pb", 90.0)],
            }),
        );

        // 注册三个，全部可见
        assert_eq!(engine.search(SearchQuery::without_limit("")).len(), 3);

        // 注销 plugin_a，应只剩 plugin_b 的 pb
        engine.unregister_plugin("plugin_a");
        let results = engine.search(SearchQuery::without_limit(""));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].provider, "pb");

        // 再次注销已不存在的插件，不应报错
        engine.unregister_plugin("plugin_a");

        // 注销 plugin_b，引擎应清空
        engine.unregister_plugin("plugin_b");
        assert!(engine.search(SearchQuery::without_limit("")).is_empty());
    }
}
