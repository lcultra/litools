use serde::Serialize;

use crate::view::{ViewProvider, WindowHostKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SurfaceLifecycle {
    Active,
    Hidden,
    Destroyed,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceMetadata {
    pub id: String,
    pub webview_label: String,
    pub view_id: String,
    pub provider: ViewProvider,
    pub route: String,
    pub title: String,
    pub host_window_label: String,
    pub host_kind: WindowHostKind,
    pub lifecycle: SurfaceLifecycle,
    pub focused: bool,
    pub created_at: String,
    pub updated_at: String,
}
