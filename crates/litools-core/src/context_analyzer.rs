use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use litools_search::{InputContext, InputDetector, SearchFeature};
use tokio_util::sync::CancellationToken;

pub struct ContextAnalyzer {
    detectors: RwLock<Vec<Arc<dyn InputDetector>>>,
}

impl ContextAnalyzer {
    pub async fn analyze(
        &self,
        input: &str,
        _cancellation: Option<&CancellationToken>,
    ) -> InputContext {
        let normalized = input.trim().to_string();

        // 快照当前 detector 列表（持读锁，不跨 await）
        let detectors = self.detectors.read().unwrap().clone();

        let features: Vec<SearchFeature> = futures::future::join_all(
            detectors.iter().map(|d| {
                let input = normalized.clone();
                let detector_id = d.id().to_string();
                let feature_kind = d.feature_kind().to_string();
                let source = d
                    .source()
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("builtin.{}", detector_id));
                async move {
                    let detection = d.detect(&input).await?;
                    Some(SearchFeature {
                        kind: feature_kind,
                        source,
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

    // ── 运行时注册（Phase 4D）──

    /// 运行时注册一个 detector（幂等替换同 id）。
    pub fn register_detector(&self, detector: Arc<dyn InputDetector>) {
        let mut detectors = self.detectors.write().unwrap();
        // 幂等 replace：同 id 移除旧版本
        detectors.retain(|d| d.id() != detector.id());
        detectors.push(detector);
    }

    /// 运行时注销一个 detector。
    pub fn unregister_detector(&self, detector_id: &str) {
        self.detectors
            .write()
            .unwrap()
            .retain(|d| d.id() != detector_id);
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
            detectors: RwLock::new(self.detectors.into_values().collect()),
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
        feature_kind: Option<&'static str>,
        source: Option<&'static str>,
        confidence: f32,
    }

    #[async_trait]
    impl InputDetector for StubDetector {
        fn id(&self) -> &str {
            self.id
        }

        fn feature_kind(&self) -> &str {
            self.feature_kind.unwrap_or(self.id)
        }

        fn source(&self) -> Option<&str> {
            self.source
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
                feature_kind: None,
                source: None,
                confidence: 0.5,
            }))
            .register(Arc::new(StubDetector {
                id: "test",
                feature_kind: None,
                source: None,
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
                feature_kind: None,
                source: None,
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
    async fn analyzer_uses_detector_feature_kind_and_source() {
        let analyzer = ContextAnalyzerBuilder::new()
            .register(Arc::new(StubDetector {
                id: "dev.plugin.detector",
                feature_kind: Some("json"),
                source: Some("plugin.dev.plugin.detector"),
                confidence: 0.8,
            }))
            .build();

        let ctx = analyzer.analyze("anything", None).await;
        assert!(ctx.has_feature("json"));
        let f = ctx.first_feature("json").unwrap();
        assert_eq!(f.kind, "json");
        assert_eq!(f.source, "plugin.dev.plugin.detector");
    }

    #[tokio::test]
    async fn empty_analyzer_returns_no_features() {
        let analyzer = ContextAnalyzerBuilder::new().build();
        let ctx = analyzer.analyze("hello", None).await;
        assert!(ctx.features.is_empty());
    }
}
