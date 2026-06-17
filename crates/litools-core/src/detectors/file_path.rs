use async_trait::async_trait;
use litools_search::{Detection, InputDetector};
use std::collections::HashMap;
use std::path::Path;

pub struct FilePathDetector;

impl FilePathDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InputDetector for FilePathDetector {
    fn id(&self) -> &str {
        "file"
    }

    async fn detect(&self, input: &str) -> Option<Detection> {
        let trimmed = input.trim();
        let is_absolute = trimmed.starts_with('/')
            || (trimmed.len() > 2
                && trimmed.as_bytes()[1] == b':'
                && (trimmed.as_bytes()[2] == b'\\' || trimmed.as_bytes()[2] == b'/'));
        if is_absolute {
            let path = Path::new(trimmed);
            let confidence = if path.exists() { 1.0 } else { 0.6 };
            return Some(Detection {
                confidence,
                metadata: HashMap::new(),
            });
        }
        if trimmed.starts_with("./") || trimmed.starts_with("../") {
            return Some(Detection {
                confidence: 0.6,
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
    async fn detects_absolute_path() {
        let d = FilePathDetector::new();
        assert!(d.detect("/usr/local/bin/test").await.is_some());
    }

    #[tokio::test]
    async fn detects_relative_path() {
        let d = FilePathDetector::new();
        assert!(d.detect("./src/main.rs").await.is_some());
    }

    #[tokio::test]
    async fn rejects_plain_text() {
        let d = FilePathDetector::new();
        assert!(d.detect("hello world").await.is_none());
    }
}
