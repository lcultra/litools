use tauri::{Emitter, Manager, Webview};

use crate::core::surface::model::SurfaceMetadata;
pub use litools_config::events::{FOCUS_SEARCH_EVENT, SURFACE_METADATA_CHANGED_EVENT};

pub fn emit_metadata_changed(app: &tauri::AppHandle, metadata: &SurfaceMetadata) {
    if let Some(webview) = app.get_webview(&metadata.webview_label) {
        let _ = webview.emit(SURFACE_METADATA_CHANGED_EVENT, metadata.clone());
    }
}

pub fn emit_focus_search(webview: &Webview) {
    let _ = webview.emit(FOCUS_SEARCH_EVENT, ());
}
