use async_trait::async_trait;
use litools_search::{Detection, InputDetector};
use std::collections::HashMap;

pub struct JsonDetector;

impl JsonDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InputDetector for JsonDetector {
    fn id(&self) -> &str {
        "json"
    }

    async fn detect(&self, input: &str) -> Option<Detection> {
        let trimmed = input.trim();
        if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
            return None;
        }
        serde_json::from_str::<serde_json::Value>(trimmed).ok()?;
        Some(Detection {
            confidence: 1.0,
            metadata: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn detects_valid_json_object() {
        let d = JsonDetector::new();
        assert!(d.detect(r#"{"key": "value"}"#).await.is_some());
    }

    #[tokio::test]
    async fn detects_valid_json_array() {
        let d = JsonDetector::new();
        assert!(d.detect(r#"[1, 2, 3]"#).await.is_some());
    }

    #[tokio::test]
    async fn rejects_plain_text() {
        let d = JsonDetector::new();
        assert!(d.detect("hello world").await.is_none());
    }

    #[tokio::test]
    async fn rejects_invalid_json() {
        let d = JsonDetector::new();
        assert!(d.detect("{invalid").await.is_none());
    }
}
