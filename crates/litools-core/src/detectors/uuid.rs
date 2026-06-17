use async_trait::async_trait;
use litools_search::{Detection, InputDetector};
use std::collections::HashMap;

pub struct UuidDetector;

impl UuidDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InputDetector for UuidDetector {
    fn id(&self) -> &str {
        "uuid"
    }

    async fn detect(&self, input: &str) -> Option<Detection> {
        let trimmed = input.trim();
        if trimmed.len() == 36
            && trimmed.chars().filter(|&c| c == '-').count() == 4
            && uuid::Uuid::parse_str(trimmed).is_ok()
        {
            return Some(Detection {
                confidence: 1.0,
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
    async fn detects_valid_uuid() {
        let d = UuidDetector::new();
        assert!(d
            .detect("550e8400-e29b-41d4-a716-446655440000")
            .await
            .is_some());
    }

    #[tokio::test]
    async fn rejects_invalid_uuid() {
        let d = UuidDetector::new();
        assert!(d.detect("not-a-uuid").await.is_none());
    }
}
