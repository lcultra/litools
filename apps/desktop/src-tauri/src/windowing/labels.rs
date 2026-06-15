pub use litools_config::labels::{DETACH_WINDOW_PREFIX, MAIN_WINDOW_LABEL};

use uuid::Uuid;

pub fn is_detach_window_label(label: &str) -> bool {
    label.starts_with(DETACH_WINDOW_PREFIX)
}

/// 生成分离窗口标签：`detach-window-{uuid}`。
pub fn detach_window_label() -> String {
    format!("{DETACH_WINDOW_PREFIX}{}", Uuid::new_v4())
}

/// 生成 Core provider 的 surface / webview 标签：`core-webview-{uuid}`。
pub fn core_webview_label() -> String {
    let prefix = litools_config::labels::CORE_WEBVIEW_PREFIX;
    format!("{prefix}{}", Uuid::new_v4())
}

/// 生成 Plugin provider 的 surface / webview 标签：`plugin-webview-{uuid}`。
pub fn plugin_webview_label() -> String {
    let prefix = litools_config::labels::PLUGIN_WEBVIEW_PREFIX;
    format!("{prefix}{}", Uuid::new_v4())
}
