use async_trait::async_trait;
use litools_search::{Detection, InputDetector};
use std::collections::HashMap;

pub struct MarkdownDetector;

impl MarkdownDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InputDetector for MarkdownDetector {
    fn id(&self) -> &str {
        "markdown"
    }

    async fn detect(&self, input: &str) -> Option<Detection> {
        let trimmed = input.trim();
        if trimmed.starts_with('[') && trimmed.contains("](") && trimmed.ends_with(')') {
            return Some(Detection {
                confidence: 0.9,
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
    async fn detects_markdown_link() {
        let d = MarkdownDetector::new();
        assert!(d
            .detect("[GitHub](https://github.com)")
            .await
            .is_some());
    }

    #[tokio::test]
    async fn rejects_plain_text() {
        let d = MarkdownDetector::new();
        assert!(d.detect("hello world").await.is_none());
    }
}
