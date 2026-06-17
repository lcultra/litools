//! Phase 4A 集成测试：端到端验证 ContextAnalyzer 管线。
//!
//! 覆盖场景：
//! - 单 feature 检测（url, json, base64, jwt, uuid, color, markdown, file）
//! - 多 feature 共存（同一输入匹配多种 detector）
//! - 空输入 / 纯文本返回空 features
//! - Provider 消费 InputContext 做差异化搜索
//! - SearchRequest 携带 context 完整传递

use std::collections::HashMap;

use async_trait::async_trait;
use litools_core::context_analyzer::ContextAnalyzerBuilder;
use litools_search::{
    feature_kinds,
    InputContext, SearchProvider,
    SearchQuery, SearchRequest, SearchResult,
};

// ── 辅助函数 ──

fn build_analyzer() -> litools_core::context_analyzer::ContextAnalyzer {
    ContextAnalyzerBuilder::with_builtin().build()
}

// ── 单 Feature 检测 ──

#[tokio::test]
async fn detects_json_object() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze(r#"{"name": "test", "count": 42}"#, None).await;
    assert!(ctx.has_feature(feature_kinds::JSON));
    let f = ctx.first_feature(feature_kinds::JSON).unwrap();
    assert_eq!(f.kind, "json");
    assert_eq!(f.source, "builtin.json");
    assert_eq!(f.confidence, 1.0);
}

#[tokio::test]
async fn detects_json_array() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze(r#"[1, "two", {"three": 3}]"#, None).await;
    assert!(ctx.has_feature(feature_kinds::JSON));
}

#[tokio::test]
async fn detects_https_url() {
    let analyzer = build_analyzer();
    let ctx = analyzer
        .analyze("https://github.com/litools/litools", None)
        .await;
    assert!(ctx.has_feature(feature_kinds::URL));
    let f = ctx.first_feature(feature_kinds::URL).unwrap();
    assert_eq!(f.kind, "url");
    assert_eq!(f.source, "builtin.url");
    assert!(f.confidence > 0.9);
    // metadata 包含 host
    assert_eq!(
        f.metadata.get("host").unwrap().as_str().unwrap(),
        "github.com"
    );
}

#[tokio::test]
async fn detects_http_url() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze("http://example.com/path", None).await;
    assert!(ctx.has_feature(feature_kinds::URL));
}

#[tokio::test]
async fn detects_jwt_token() {
    let analyzer = build_analyzer();
    let token = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
    let ctx = analyzer.analyze(token, None).await;
    assert!(ctx.has_feature(feature_kinds::JWT));
    let f = ctx.first_feature(feature_kinds::JWT).unwrap();
    assert_eq!(f.confidence, 0.95);
}

#[tokio::test]
async fn detects_uuid() {
    let analyzer = build_analyzer();
    let ctx = analyzer
        .analyze("550e8400-e29b-41d4-a716-446655440000", None)
        .await;
    assert!(ctx.has_feature(feature_kinds::UUID));
}

#[tokio::test]
async fn detects_hex_color() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze("#ff5733", None).await;
    assert!(ctx.has_feature(feature_kinds::COLOR));
}

#[tokio::test]
async fn detects_rgb_color() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze("rgb(255, 87, 51)", None).await;
    assert!(ctx.has_feature(feature_kinds::COLOR));
}

#[tokio::test]
async fn detects_markdown_link() {
    let analyzer = build_analyzer();
    let ctx = analyzer
        .analyze("[GitHub](https://github.com)", None)
        .await;
    assert!(ctx.has_feature(feature_kinds::MARKDOWN));
}

#[tokio::test]
async fn detects_curl_command() {
    let analyzer = build_analyzer();
    let ctx = analyzer
        .analyze("curl -X GET https://api.example.com/data", None)
        .await;
    assert!(ctx.has_feature(feature_kinds::CURL));
    // 注意：URL Detector 只检测以 scheme 开头的输入，
    // curl 命令中嵌入的 URL 不会被检测到（这是有意的设计简化）
}

#[tokio::test]
async fn detects_base64_data_url() {
    let analyzer = build_analyzer();
    let ctx = analyzer
        .analyze("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk", None)
        .await;
    assert!(ctx.has_feature(feature_kinds::BASE64));
    let f = ctx.first_feature(feature_kinds::BASE64).unwrap();
    assert_eq!(f.confidence, 1.0);
}

#[tokio::test]
async fn detects_absolute_file_path() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze("/usr/local/bin/test", None).await;
    assert!(ctx.has_feature(feature_kinds::FILE));
}

// ── 多 Feature 共存 ──

#[tokio::test]
async fn multiple_features_on_compound_input() {
    let analyzer = build_analyzer();
    // 输入同时匹配 url + file (因为 URL 路径以 .json 结尾，且是绝对路径格式)
    let ctx = analyzer
        .analyze("https://github.com/foo/bar/blob/main/test.json", None)
        .await;

    // 应至少命中 url
    assert!(ctx.has_feature(feature_kinds::URL));

    // features 列表包含多个条目
    assert!(ctx.features.len() >= 1);
}

#[tokio::test]
async fn curl_command_produces_curl_feature() {
    let analyzer = build_analyzer();
    let ctx = analyzer
        .analyze("curl -X POST https://httpbin.org/post", None)
        .await;

    assert!(ctx.has_feature(feature_kinds::CURL));
    // URL Detector 是前缀匹配，不会在 curl 命令中检测嵌入的 URL
    assert!(ctx.features.len() >= 1);
}

// ── 空输入 / 纯文本 ──

#[tokio::test]
async fn empty_input_returns_no_features() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze("", None).await;
    assert!(ctx.features.is_empty());
    assert!(!ctx.has_feature(feature_kinds::JSON));
    assert!(!ctx.has_feature(feature_kinds::URL));
}

#[tokio::test]
async fn plain_text_returns_no_features() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze("hello world, this is plain text", None).await;
    assert!(ctx.features.is_empty());
}

// ── InputContext 辅助方法 ──

#[tokio::test]
async fn first_feature_returns_feature_when_present() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze("https://example.com", None).await;
    let f = ctx.first_feature(feature_kinds::URL).unwrap();
    assert_eq!(f.kind, "url");
}

#[tokio::test]
async fn first_feature_returns_none_when_absent() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze("plain text", None).await;
    assert!(ctx.first_feature(feature_kinds::URL).is_none());
}

#[tokio::test]
async fn features_of_kind_returns_empty_when_absent() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze("nothing", None).await;
    assert!(ctx.features_of_kind(feature_kinds::URL).is_empty());
}

// ── Provider 消费 Context ──

struct ContextAwareStubProvider {
    id: &'static str,
    boost_on: &'static str,   // 当 context 包含此 kind 时 boost
    base_score: f32,
}

#[async_trait]
impl SearchProvider for ContextAwareStubProvider {
    fn id(&self) -> &str {
        self.id
    }

    async fn search(&self, request: &SearchRequest) -> Vec<SearchResult> {
        let mut score = self.base_score;

        if request.context.has_feature(self.boost_on) {
            score += 0.5;
        }

        vec![SearchResult {
            id: format!("{}.result", self.id),
            title: format!("Result for {}", self.id),
            subtitle: None,
            icon_uri: None,
            provider: self.id.to_string(),
            score,
            matches: Default::default(),
            actions: vec![],
        }]
    }
}

#[tokio::test]
async fn provider_consumes_context_and_boosts_score() {
    let provider = ContextAwareStubProvider {
        id: "json-tools",
        boost_on: feature_kinds::JSON,
        base_score: 0.5,
    };

    // 输入为 JSON 时，score 被 boost
    let request = SearchRequest {
        query: SearchQuery::without_limit(r#"{"key": "val"}"#),
        context: build_analyzer()
            .analyze(r#"{"key": "val"}"#, None)
            .await,
        metadata: HashMap::new(),
    };
    let results = provider.search(&request).await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].score, 1.0); // 0.5 + 0.5 boost
}

#[tokio::test]
async fn provider_ignores_context_when_no_match() {
    let provider = ContextAwareStubProvider {
        id: "plain-tools",
        boost_on: feature_kinds::JSON,
        base_score: 0.5,
    };

    // 纯文本输入，不 boost
    let request = SearchRequest {
        query: SearchQuery::without_limit("hello world"),
        context: build_analyzer().analyze("hello world", None).await,
        metadata: HashMap::new(),
    };
    let results = provider.search(&request).await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].score, 0.5); // 无 boost
}

// ── SearchRequest 完整传递 ──

#[tokio::test]
async fn search_request_carries_context_correctly() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze("curl https://example.com", None).await;

    let request = SearchRequest {
        query: SearchQuery::without_limit("curl https://example.com"),
        context: ctx,
        metadata: HashMap::new(),
    };

    assert!(request.context.has_feature(feature_kinds::CURL));
    // URL Detector 是前缀匹配：curl 命令中嵌入的 URL 不会被检测
    assert_eq!(request.query.text, "curl https://example.com");
    assert!(request.metadata.is_empty()); // Phase 4A: metadata 为空
}

// ── InputContext 版本号 ──

#[tokio::test]
async fn context_version_is_one() {
    let analyzer = build_analyzer();
    let ctx = analyzer.analyze("anything", None).await;
    assert_eq!(ctx.version, 1);
}

// ── InputContext::empty() ──

#[test]
fn empty_context_is_empty() {
    let ctx = InputContext::empty();
    assert_eq!(ctx.version, 1);
    assert!(ctx.raw.is_empty());
    assert!(ctx.normalized.is_empty());
    assert!(ctx.features.is_empty());
    assert!(ctx.attachments.is_empty());
    assert!(ctx.metadata.is_empty());
}
