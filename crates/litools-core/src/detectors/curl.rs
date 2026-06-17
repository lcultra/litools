use async_trait::async_trait;
use litools_search::{Detection, InputDetector};
use std::collections::HashMap;

pub struct CurlDetector;

impl CurlDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InputDetector for CurlDetector {
    fn id(&self) -> &str {
        "curl"
    }

    async fn detect(&self, input: &str) -> Option<Detection> {
        if input.trim().starts_with("curl ") {
            return Some(Detection {
                confidence: 0.95,
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
    async fn detects_curl_command() {
        let d = CurlDetector::new();
        assert!(d
            .detect("curl -X GET https://api.example.com")
            .await
            .is_some());
    }

    #[tokio::test]
    async fn rejects_plain_text() {
        let d = CurlDetector::new();
        assert!(d.detect("hello world").await.is_none());
    }
}
