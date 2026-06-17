use async_trait::async_trait;
use litools_search::{Detection, InputDetector};
use std::collections::HashMap;

pub struct UrlDetector;

impl UrlDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InputDetector for UrlDetector {
    fn id(&self) -> &str {
        "url"
    }

    async fn detect(&self, input: &str) -> Option<Detection> {
        let trimmed = input.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            if let Ok(parsed) = url::Url::parse(trimmed) {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "scheme".to_string(),
                    serde_json::Value::String(parsed.scheme().to_string()),
                );
                if let Some(host) = parsed.host_str() {
                    metadata.insert(
                        "host".to_string(),
                        serde_json::Value::String(host.to_string()),
                    );
                }
                return Some(Detection {
                    confidence: 0.95,
                    metadata,
                });
            }
        }
        if trimmed.starts_with("ftp://")
            || trimmed.starts_with("file://")
            || trimmed.starts_with("ws://")
            || trimmed.starts_with("wss://")
        {
            return Some(Detection {
                confidence: 0.7,
                metadata: HashMap::new(),
            });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn detects_https_url() {
        let d = UrlDetector::new();
        let detection = d.detect("https://github.com/foo/bar").await.unwrap();
        assert!(detection.confidence > 0.9);
        assert_eq!(
            detection.metadata.get("host").unwrap().as_str().unwrap(),
            "github.com"
        );
    }

    #[tokio::test]
    async fn rejects_plain_text() {
        let d = UrlDetector::new();
        assert!(d.detect("hello world").await.is_none());
    }
}
