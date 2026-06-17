use async_trait::async_trait;
use litools_search::{Detection, InputDetector};
use std::collections::HashMap;

pub struct JwtDetector;

impl JwtDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InputDetector for JwtDetector {
    fn id(&self) -> &str {
        "jwt"
    }

    async fn detect(&self, input: &str) -> Option<Detection> {
        let trimmed = input.trim();
        let parts: Vec<&str> = trimmed.split('.').collect();
        if parts.len() == 3
            && !trimmed.contains(' ')
            && parts.iter().all(|p| {
                !p.is_empty()
                    && p.chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            })
        {
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
    async fn detects_jwt_token() {
        let d = JwtDetector::new();
        assert!(d
            .detect(
                "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"
            )
            .await
            .is_some());
    }

    #[tokio::test]
    async fn rejects_plain_text() {
        let d = JwtDetector::new();
        assert!(d.detect("hello world").await.is_none());
    }
}
