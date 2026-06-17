use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use crate::input::InputContext;
use crate::request::SearchRequest;
use crate::{
    SearchResult,
    provider::SearchProvider,
    query::SearchQuery,
    ranking::rank_results,
};

#[derive(Default)]
pub struct SearchEngine {
    providers: RwLock<Vec<Arc<dyn SearchProvider>>>,
    plugin_providers: RwLock<HashMap<String, Vec<String>>>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个无主搜索提供者（内部归入 `__builtin__` 分组）。
    pub fn register_provider(&self, provider: Arc<dyn SearchProvider>) {
        self.register_plugin_provider("__builtin__", provider);
    }

    /// 注册属于指定插件的搜索提供者。
    pub fn register_plugin_provider(&self, owner_plugin_id: &str, provider: Arc<dyn SearchProvider>) {
        self.plugin_providers
            .write()
            .unwrap()
            .entry(owner_plugin_id.to_string())
            .or_default()
            .push(provider.id().to_string());
        self.providers.write().unwrap().push(provider);
    }

    /// 注销指定插件拥有的所有搜索提供者。
    pub fn unregister_plugin(&self, plugin_id: &str) {
        if let Some(provider_ids) = self.plugin_providers.write().unwrap().remove(plugin_id) {
            self.providers
                .write()
                .unwrap()
                .retain(|p| !provider_ids.contains(&p.id().to_string()));
        }
    }

    /// 按 provider_id 注销单个 provider。
    pub fn unregister_provider(&self, provider_id: &str) {
        self.providers.write().unwrap().retain(|p| p.id() != provider_id);
        if let Ok(mut pp) = self.plugin_providers.write() {
            for ids in pp.values_mut() {
                ids.retain(|id| id != provider_id);
            }
        }
    }

    pub async fn search(&self, query: SearchQuery) -> Vec<SearchResult> {
        self.search_with_providers(query, std::iter::empty::<&str>())
            .await
    }

    #[allow(clippy::needless_lifetimes)]
    pub async fn search_with_providers<'a>(
        &self,
        query: SearchQuery,
        enabled_provider_ids: impl IntoIterator<Item = &'a str>,
    ) -> Vec<SearchResult> {
        let request = SearchRequest {
            query,
            context: InputContext::empty(),
            metadata: HashMap::new(),
        };
        self.search_with_request(&request, enabled_provider_ids)
            .await
    }

    pub async fn search_with_request<'a>(
        &self,
        request: &SearchRequest,
        enabled_provider_ids: impl IntoIterator<Item = &'a str>,
    ) -> Vec<SearchResult> {
        let enabled_provider_ids = enabled_provider_ids.into_iter().collect::<Vec<_>>();

        let providers = self
            .providers
            .read()
            .unwrap()
            .iter()
            .filter(|p| {
                enabled_provider_ids.is_empty() || enabled_provider_ids.contains(&p.id())
            })
            .cloned()
            .collect::<Vec<_>>();

        let cancel = CancellationToken::new();
        let mut set = tokio::task::JoinSet::new();

        for provider in providers {
            let cancel = cancel.clone();
            let request = request.clone();

            set.spawn(async move {
                tokio::select! {
                    results = provider.search(&request) => results,
                    _ = cancel.cancelled() => vec![],
                }
            });
        }

        let mut results = Vec::new();
        while let Some(res) = set.join_next().await {
            if let Ok(provider_results) = res {
                results.extend(provider_results);
            }
        }

        rank_results(results, request.query.limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::request::SearchRequest;
    use crate::SearchResultAction;

    struct StubProvider {
        id: &'static str,
        results: Vec<SearchResult>,
    }

    #[async_trait]
    impl SearchProvider for StubProvider {
        fn id(&self) -> &str {
            self.id
        }

        async fn search(&self, _request: &SearchRequest) -> Vec<SearchResult> {
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

    #[tokio::test]
    async fn empty_engine_returns_empty_results() {
        let engine = SearchEngine::new();
        let results = engine.search(SearchQuery::with_limit("test", 10)).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn single_provider_searches() {
        let engine = SearchEngine::new();
        engine.register_provider(Arc::new(StubProvider {
            id: "test",
            results: vec![result_for("Alpha", "test", 100.0)],
        }));

        let results = engine.search(SearchQuery::without_limit("alpha")).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Alpha");
    }

    #[tokio::test]
    async fn multiple_providers_are_merged() {
        let engine = SearchEngine::new();
        engine.register_provider(Arc::new(StubProvider {
            id: "a",
            results: vec![result_for("A1", "a", 100.0)],
        }));
        engine.register_provider(Arc::new(StubProvider {
            id: "b",
            results: vec![result_for("B1", "b", 90.0)],
        }));

        let results = engine.search(SearchQuery::without_limit("")).await;
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].provider, "a");
        assert_eq!(results[1].provider, "b");
    }

    #[tokio::test]
    async fn enabled_provider_ids_filter() {
        let engine = SearchEngine::new();
        engine.register_provider(Arc::new(StubProvider {
            id: "a",
            results: vec![result_for("A1", "a", 100.0)],
        }));
        engine.register_provider(Arc::new(StubProvider {
            id: "b",
            results: vec![result_for("B1", "b", 90.0)],
        }));

        let results = engine
            .search_with_providers(SearchQuery::without_limit(""), ["a"].iter().copied())
            .await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].provider, "a");
    }

    #[tokio::test]
    async fn search_respects_limit() {
        let engine = SearchEngine::new();
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

        let results = engine.search(SearchQuery::with_limit("", 3)).await;
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn register_plugin_provider_and_unregister() {
        let engine = SearchEngine::new();
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

        assert_eq!(engine.search(SearchQuery::without_limit("")).await.len(), 3);

        engine.unregister_plugin("plugin_a");
        let results = engine.search(SearchQuery::without_limit("")).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].provider, "pb");

        engine.unregister_plugin("plugin_a");
        engine.unregister_plugin("plugin_b");
        assert!(engine.search(SearchQuery::without_limit("")).await.is_empty());
    }

    #[tokio::test]
    async fn unregister_single_provider() {
        let engine = SearchEngine::new();
        engine.register_provider(Arc::new(StubProvider {
            id: "a",
            results: vec![result_for("A1", "a", 100.0)],
        }));
        engine.register_provider(Arc::new(StubProvider {
            id: "b",
            results: vec![result_for("B1", "b", 90.0)],
        }));

        engine.unregister_provider("a");
        let results = engine.search(SearchQuery::without_limit("")).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].provider, "b");
    }
}
