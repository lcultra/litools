use serde::{Deserialize, Serialize};

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

pub trait SearchProvider: Send + Sync {
    fn id(&self) -> &'static str;

    fn search(&self, query: &SearchQuery) -> Vec<SearchResult>;
}
