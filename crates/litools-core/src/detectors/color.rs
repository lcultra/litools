use async_trait::async_trait;
use litools_search::{Detection, InputDetector};
use std::collections::HashMap;

pub struct ColorDetector;

impl ColorDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InputDetector for ColorDetector {
    fn id(&self) -> &str {
        "color"
    }

    async fn detect(&self, input: &str) -> Option<Detection> {
        let trimmed = input.trim();
        if trimmed.starts_with('#')
            && (trimmed.len() == 4 || trimmed.len() == 7 || trimmed.len() == 9)
            && trimmed[1..]
                .chars()
                .all(|c| c.is_ascii_hexdigit())
        {
            return Some(Detection {
                confidence: 1.0,
                metadata: HashMap::new(),
            });
        }
        if (trimmed.starts_with("rgb(")
            || trimmed.starts_with("rgba(")
            || trimmed.starts_with("hsl("))
            && trimmed.ends_with(')')
        {
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
    async fn detects_hex_color() {
        let d = ColorDetector::new();
        assert!(d.detect("#ff5733").await.is_some());
    }

    #[tokio::test]
    async fn detects_rgb() {
        let d = ColorDetector::new();
        assert!(d.detect("rgb(255, 87, 51)").await.is_some());
    }

    #[tokio::test]
    async fn rejects_plain_text() {
        let d = ColorDetector::new();
        assert!(d.detect("hello world").await.is_none());
    }
}
