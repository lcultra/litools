use tauri::{Emitter, Manager, WebviewWindow};

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const FOCUS_SEARCH_EVENT: &str = "focus-search";

pub fn main_window(app: &tauri::AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(MAIN_WINDOW_LABEL)
}

pub fn show_main_window(window: &WebviewWindow) {
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.emit(FOCUS_SEARCH_EVENT, ());
}

pub fn hide_main_window(window: &WebviewWindow) {
    let _ = window.hide();
}

pub fn toggle_main_window(window: &WebviewWindow) {
    if window.is_visible().unwrap_or(false) {
        hide_main_window(window);
    } else {
        show_main_window(window);
    }
}
