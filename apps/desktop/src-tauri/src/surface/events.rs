use tauri::{Emitter, Manager, Webview};

use crate::surface::model::SurfaceMetadata;

pub const FOCUS_SEARCH_EVENT: &str = "focus-search";
pub const NAVIGATE_EVENT: &str = "navigate";
pub const SURFACE_METADATA_CHANGED_EVENT: &str = "surface-metadata-changed";

pub fn emit_metadata_changed(app: &tauri::AppHandle, metadata: &SurfaceMetadata) {
    if let Some(webview) = app.get_webview(&metadata.webview_label) {
        let _ = webview.emit(SURFACE_METADATA_CHANGED_EVENT, metadata.clone());
    }
}

pub fn emit_navigate(webview: &Webview, route: &str) {
    let _ = webview.emit(NAVIGATE_EVENT, route);
}

pub fn emit_focus_search(webview: &Webview) {
    let _ = webview.emit(FOCUS_SEARCH_EVENT, ());
}
