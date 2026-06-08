pub const MAIN_WINDOW_LABEL: &str = "main";
pub const DETACHED_PANEL_WINDOW_PREFIX: &str = "detached-panel-";
const SURFACE_WEBVIEW_LABEL_PREFIX: &str = "surface-";

pub fn is_detached_panel_window_label(label: &str) -> bool {
    label.starts_with(DETACHED_PANEL_WINDOW_PREFIX)
}

pub fn surface_webview_label(surface_id: &str) -> String {
    format!("{SURFACE_WEBVIEW_LABEL_PREFIX}{surface_id}")
}
