use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{matcher::SearchResultMatches, query::SearchQuery};

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

/// 搜索上下文 —— 所有 Provider 通用的搜索元信息
#[derive(Clone)]
pub struct SearchContext {
    pub cancel: CancellationToken,
    pub timeout: Duration,
    pub trace_id: Uuid,
}

impl SearchContext {
    pub fn new(timeout: Duration) -> Self {
        Self {
            cancel: CancellationToken::new(),
            timeout,
            trace_id: Uuid::new_v4(),
        }
    }
}

#[async_trait]
pub trait SearchProvider: Send + Sync {
    fn id(&self) -> &str;

    /// 每个 Provider 可自定义超时（默认 300ms）
    fn timeout(&self) -> Duration {
        Duration::from_millis(300)
    }

    async fn search(&self, query: &SearchQuery, ctx: SearchContext) -> Vec<SearchResult>;
}
