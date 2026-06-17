use async_trait::async_trait;
use litools_search::{Detection, InputDetector};
use std::collections::HashMap;

pub struct Base64Detector;

impl Base64Detector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InputDetector for Base64Detector {
    fn id(&self) -> &str {
        "base64"
    }

    async fn detect(&self, input: &str) -> Option<Detection> {
        let trimmed = input.trim();
        if trimmed.starts_with("data:") {
            return Some(Detection {
                confidence: 1.0,
                metadata: HashMap::new(),
            });
        }
        let stripped: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
        if stripped.len() >= 20
            && stripped.len() % 4 == 0
            && stripped
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        {
            return Some(Detection {
                confidence: 0.8,
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
    async fn detects_data_url() {
        let d = Base64Detector::new();
        let r = d
            .detect("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk")
            .await
            .unwrap();
        assert_eq!(r.confidence, 1.0);
    }

    #[tokio::test]
    async fn rejects_plain_text() {
        let d = Base64Detector::new();
        assert!(d.detect("hello world").await.is_none());
    }
}
