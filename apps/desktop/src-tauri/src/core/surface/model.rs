use serde::{Deserialize, Serialize};

use crate::view::{ViewProvider, WindowHostKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SurfaceLifecycle {
    Active,
    Hidden,
    Destroyed,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
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
    pub bounds: Option<SurfaceBounds>,
    pub lifecycle: SurfaceLifecycle,
    pub focused: bool,
    pub created_at: String,
    pub updated_at: String,
}
