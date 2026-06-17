use std::sync::Arc;

use indexmap::IndexMap;
use litools_search::{InputContext, InputDetector, SearchFeature};
use tokio_util::sync::CancellationToken;

pub struct ContextAnalyzer {
    detectors: Vec<Arc<dyn InputDetector>>,
}

impl ContextAnalyzer {
    pub async fn analyze(
        &self,
        input: &str,
        _cancellation: Option<&CancellationToken>,
    ) -> InputContext {
        let normalized = input.trim().to_string();

        let features: Vec<SearchFeature> = futures::future::join_all(
            self.detectors.iter().map(|d| {
                let input = normalized.clone();
                async move {
                    let detection = d.detect(&input).await?;
                    Some(SearchFeature {
                        kind: d.id().to_string(),
                        source: format!("builtin.{}", d.id()),
                        confidence: detection.confidence,
                        metadata: detection.metadata,
                    })
                }
            }),
        )
        .await
        .into_iter()
        .flatten()
        .collect();

        InputContext {
            version: 1,
            raw: input.to_string(),
            normalized,
            features,
            attachments: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }
}

// ── Builder ──

pub struct ContextAnalyzerBuilder {
    detectors: IndexMap<String, Arc<dyn InputDetector>>,
}

impl ContextAnalyzerBuilder {
    pub fn new() -> Self {
        Self {
            detectors: IndexMap::new(),
        }
    }

    pub fn with_builtin() -> Self {
        Self::new()
            .register(Arc::new(crate::detectors::json::JsonDetector::new()))
            .register(Arc::new(crate::detectors::url::UrlDetector::new()))
            .register(Arc::new(crate::detectors::base64::Base64Detector::new()))
            .register(Arc::new(crate::detectors::file_path::FilePathDetector::new()))
            .register(Arc::new(crate::detectors::curl::CurlDetector::new()))
            .register(Arc::new(crate::detectors::jwt::JwtDetector::new()))
            .register(Arc::new(crate::detectors::uuid::UuidDetector::new()))
            .register(Arc::new(crate::detectors::color::ColorDetector::new()))
            .register(Arc::new(crate::detectors::markdown::MarkdownDetector::new()))
    }

    pub fn register(mut self, detector: Arc<dyn InputDetector>) -> Self {
        self.detectors.insert(detector.id().to_string(), detector);
        self
    }

    pub fn build(self) -> ContextAnalyzer {
        ContextAnalyzer {
            detectors: self.detectors.into_values().collect(),
        }
    }
}

impl Default for ContextAnalyzerBuilder {
    fn default() -> Self {
        Self::with_builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use litools_search::Detection;

    struct StubDetector {
        id: &'static str,
        confidence: f32,
    }

    #[async_trait]
    impl InputDetector for StubDetector {
        fn id(&self) -> &str {
            self.id
        }

        async fn detect(&self, _input: &str) -> Option<Detection> {
            Some(Detection {
                confidence: self.confidence,
                metadata: std::collections::HashMap::new(),
            })
        }
    }

    #[tokio::test]
    async fn builder_replace_same_id() {
        let analyzer = ContextAnalyzerBuilder::new()
            .register(Arc::new(StubDetector {
                id: "test",
                confidence: 0.5,
            }))
            .register(Arc::new(StubDetector {
                id: "test",
                confidence: 0.9,
            }))
            .build();

        let ctx = analyzer.analyze("hello", None).await;
        let feats = ctx.features_of_kind("test");
        assert_eq!(feats.len(), 1);
        assert_eq!(feats[0].confidence, 0.9);
    }

    #[tokio::test]
    async fn analyze_fills_kind_and_source() {
        let analyzer = ContextAnalyzerBuilder::new()
            .register(Arc::new(StubDetector {
                id: "mock",
                confidence: 0.8,
            }))
            .build();

        let ctx = analyzer.analyze("anything", None).await;
        assert!(ctx.has_feature("mock"));
        let f = ctx.first_feature("mock").unwrap();
        assert_eq!(f.kind, "mock");
        assert_eq!(f.source, "builtin.mock");
    }

    #[tokio::test]
    async fn empty_analyzer_returns_no_features() {
        let analyzer = ContextAnalyzerBuilder::new().build();
        let ctx = analyzer.analyze("hello", None).await;
        assert!(ctx.features.is_empty());
    }
}
