use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{matcher::SearchResultMatches, request::SearchRequest};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon_uri: Option<String>,
    pub provider: String,
    pub score: f32,
    #[serde(default)]
    pub matches: SearchResultMatches,
    pub actions: Vec<SearchResultAction>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchResultAction {
    pub id: String,
    pub label: String,
}

#[async_trait]
pub trait SearchProvider: Send + Sync {
    fn id(&self) -> &str;

    /// 每个 Provider 可自定义超时（默认 300ms）
    fn timeout(&self) -> Duration {
        Duration::from_millis(300)
    }

    async fn search(&self, request: &SearchRequest) -> Vec<SearchResult>;
}
