use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub confidence: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[async_trait]
pub trait InputDetector: Send + Sync {
    fn id(&self) -> &str;
    async fn detect(&self, input: &str) -> Option<Detection>;
}
