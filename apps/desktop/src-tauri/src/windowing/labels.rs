pub const MAIN_WINDOW_LABEL: &str = "main";
pub const DETACHED_PANEL_WINDOW_PREFIX: &str = "detached-panel-";
pub const PLUGIN_RUNTIME_WINDOW_PREFIX: &str = "plugin-runtime-window-";
pub const PLUGIN_RUNTIME_HEADER_WEBVIEW_PREFIX: &str = "plugin-runtime-header-";
pub const PLUGIN_RUNTIME_WEBVIEW_PREFIX: &str = "plugin-runtime-";
const SURFACE_WEBVIEW_LABEL_PREFIX: &str = "surface-";

pub fn is_detached_panel_window_label(label: &str) -> bool {
    label.starts_with(DETACHED_PANEL_WINDOW_PREFIX)
}

pub fn is_plugin_runtime_window_label(label: &str) -> bool {
    label.starts_with(PLUGIN_RUNTIME_WINDOW_PREFIX)
}

pub fn is_plugin_runtime_webview_label(label: &str) -> bool {
    label.starts_with(PLUGIN_RUNTIME_WEBVIEW_PREFIX)
        && !label.starts_with(PLUGIN_RUNTIME_HEADER_WEBVIEW_PREFIX)
}

pub fn surface_webview_label(surface_id: &str) -> String {
    format!("{SURFACE_WEBVIEW_LABEL_PREFIX}{surface_id}")
}

pub fn plugin_runtime_window_label(runtime_id: &str) -> String {
    format!("{PLUGIN_RUNTIME_WINDOW_PREFIX}{runtime_id}")
}

pub fn plugin_runtime_webview_label(runtime_id: &str) -> String {
    format!("{PLUGIN_RUNTIME_WEBVIEW_PREFIX}{runtime_id}")
}

pub fn plugin_runtime_header_webview_label(runtime_id: &str) -> String {
    format!("{PLUGIN_RUNTIME_HEADER_WEBVIEW_PREFIX}{runtime_id}")
}
