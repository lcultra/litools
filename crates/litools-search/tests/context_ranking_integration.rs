//! Phase 4B 集成测试：Context-Aware Ranking。
//!
//! 验证：
//! - 声明 supports 的 Provider 在匹配 context feature 时获得加权
//! - 未声明 supports 的 Provider 不受影响
//! - 多个 feature 匹配时 Weight 叠加
//! - SearchEngine 端到端：InputContext → boost → 排序

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use litools_search::{
    SearchEngine, SearchProvider, SearchQuery, SearchRequest,
    SearchResult, SearchResultAction, InputContext, SearchFeature,
};

fn make_feature(kind: &str) -> SearchFeature {
    SearchFeature {
        kind: kind.to_string(),
        source: format!("builtin.{}", kind),
        confidence: 1.0,
        metadata: HashMap::new(),
    }
}

fn make_result(provider: &str, title: &str, score: f32) -> SearchResult {
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

// ── 带 supports 声明的 Provider ──

struct JsonAffinityProvider;

#[async_trait]
impl SearchProvider for JsonAffinityProvider {
    fn id(&self) -> &str {
        "json-tools"
    }

    fn supports(&self) -> &[&str] {
        &["json"]
    }

    async fn search(&self, _request: &SearchRequest) -> Vec<SearchResult> {
        vec![make_result("json-tools", "JSON Formatter", 50.0)]
    }
}

struct UrlAffinityProvider;

#[async_trait]
impl SearchProvider for UrlAffinityProvider {
    fn id(&self) -> &str {
        "url-tools"
    }

    fn supports(&self) -> &[&str] {
        &["url"]
    }

    async fn search(&self, _request: &SearchRequest) -> Vec<SearchResult> {
        vec![make_result("url-tools", "Browser Open", 50.0)]
    }
}

struct MultiAffinityProvider;

#[async_trait]
impl SearchProvider for MultiAffinityProvider {
    fn id(&self) -> &str {
        "multi-tools"
    }

    fn supports(&self) -> &[&str] {
        &["json", "url"]
    }

    async fn search(&self, _request: &SearchRequest) -> Vec<SearchResult> {
        vec![make_result("multi-tools", "Multi Tool", 50.0)]
    }
}

struct NoAffinityProvider;

#[async_trait]
impl SearchProvider for NoAffinityProvider {
    fn id(&self) -> &str {
        "no-affinity"
    }

    async fn search(&self, _request: &SearchRequest) -> Vec<SearchResult> {
        vec![make_result("no-affinity", "Generic Tool", 50.0)]
    }
}

// ── 单 Provider 加权测试 ──

#[tokio::test]
async fn json_context_boosts_json_provider() {
    let engine = SearchEngine::new();
    engine.register_provider(Arc::new(JsonAffinityProvider));
    engine.register_provider(Arc::new(NoAffinityProvider));

    let request = SearchRequest {
        query: SearchQuery::without_limit("test"),
        context: InputContext {
            features: vec![make_feature("json")],
            ..InputContext::empty()
        },
        metadata: HashMap::new(),
    };

    let results = engine
        .search_with_request(&request, std::iter::empty::<&str>())
        .await;

    // json-tools: 50 + 0.5 = 50.5 → 排第一
    // no-affinity: 50 → 排第二
    assert_eq!(results[0].provider, "json-tools");
    assert!((results[0].score - 50.5).abs() < 0.01);
    assert_eq!(results[1].provider, "no-affinity");
}

#[tokio::test]
async fn url_context_boosts_url_provider() {
    let engine = SearchEngine::new();
    engine.register_provider(Arc::new(UrlAffinityProvider));
    engine.register_provider(Arc::new(NoAffinityProvider));

    let request = SearchRequest {
        query: SearchQuery::without_limit("test"),
        context: InputContext {
            features: vec![make_feature("url")],
            ..InputContext::empty()
        },
        metadata: HashMap::new(),
    };

    let results = engine
        .search_with_request(&request, std::iter::empty::<&str>())
        .await;

    assert_eq!(results[0].provider, "url-tools");
    assert!((results[0].score - 50.5).abs() < 0.01);
}

// ── 多 Feature 叠加 ──

#[tokio::test]
async fn multiple_features_stack_boost() {
    let engine = SearchEngine::new();
    engine.register_provider(Arc::new(MultiAffinityProvider));
    engine.register_provider(Arc::new(NoAffinityProvider));

    let request = SearchRequest {
        query: SearchQuery::without_limit("test"),
        context: InputContext {
            features: vec![make_feature("json"), make_feature("url")],
            ..InputContext::empty()
        },
        metadata: HashMap::new(),
    };

    let results = engine
        .search_with_request(&request, std::iter::empty::<&str>())
        .await;

    // multi-tools: 50 + 0.5 + 0.5 = 51.0
    assert_eq!(results[0].provider, "multi-tools");
    assert!((results[0].score - 51.0).abs() < 0.01);
}

// ── 空 Context 不产生加权 ──

#[tokio::test]
async fn empty_context_no_boost() {
    let engine = SearchEngine::new();
    engine.register_provider(Arc::new(JsonAffinityProvider));
    engine.register_provider(Arc::new(NoAffinityProvider));

    let request = SearchRequest {
        query: SearchQuery::without_limit("test"),
        context: InputContext::empty(),
        metadata: HashMap::new(),
    };

    let results = engine
        .search_with_request(&request, std::iter::empty::<&str>())
        .await;

    // 分数相等时按 title 排序: "Generic Tool" < "JSON Formatter"
    // 两者都是 50.0
    assert_eq!(results[0].score, 50.0);
    assert_eq!(results[1].score, 50.0);
}

// ── 向后兼容：search_with_providers 不产生加权 ──

#[tokio::test]
async fn search_with_providers_has_no_context_boost() {
    let engine = SearchEngine::new();
    engine.register_provider(Arc::new(JsonAffinityProvider));
    engine.register_provider(Arc::new(NoAffinityProvider));

    let results = engine
        .search_with_providers(SearchQuery::without_limit("test"), std::iter::empty::<&str>())
        .await;

    // search_with_providers 构造空 InputContext，无加权
    assert_eq!(results[0].score, 50.0);
    assert_eq!(results[1].score, 50.0);
}

// ── 无关 feature 不产生加权 ──

#[tokio::test]
async fn irrelevant_feature_no_boost() {
    let engine = SearchEngine::new();
    engine.register_provider(Arc::new(JsonAffinityProvider));
    engine.register_provider(Arc::new(NoAffinityProvider));

    let request = SearchRequest {
        query: SearchQuery::without_limit("test"),
        context: InputContext {
            features: vec![make_feature("color")], // json-tools 不关心 color
            ..InputContext::empty()
        },
        metadata: HashMap::new(),
    };

    let results = engine
        .search_with_request(&request, std::iter::empty::<&str>())
        .await;

    // 两者都是 50.0，无加权
    assert_eq!(results[0].score, 50.0);
    assert_eq!(results[1].score, 50.0);
}
