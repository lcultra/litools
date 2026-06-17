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

    /// detector 产出的 feature 类型。默认与 detector id 相同，内置 detector 可保持零配置。
    fn feature_kind(&self) -> &str {
        self.id()
    }

    /// feature 的来源标识。默认由上层分析器按内置 detector 处理。
    fn source(&self) -> Option<&str> {
        None
    }

    async fn detect(&self, input: &str) -> Option<Detection>;
}
