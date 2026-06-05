use litools_search::SearchResult;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherItem {
    pub result: SearchResult,
    pub is_pinned: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherSection {
    pub id: String,
    pub title: String,
    pub items: Vec<LauncherItem>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherPanelResponse {
    pub sections: Vec<LauncherSection>,
}
