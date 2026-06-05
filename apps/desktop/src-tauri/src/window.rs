use tauri::{Emitter, LogicalSize, Manager, Size, WebviewWindow};

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const FOCUS_SEARCH_EVENT: &str = "focus-search";
pub const NAVIGATE_EVENT: &str = "navigate";

const MANAGEMENT_WINDOW_WIDTH: f64 = 820.0;
const MANAGEMENT_WINDOW_HEIGHT: f64 = 560.0;

pub fn main_window(app: &tauri::AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(MAIN_WINDOW_LABEL)
}

pub fn show_main_window(window: &WebviewWindow, center_on_show: bool) {
    if center_on_show {
        let _ = window.center();
    }
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.emit(FOCUS_SEARCH_EVENT, ());
}

pub fn open_route(window: &WebviewWindow, route: &str, _center_on_show: bool) {
    if route == "/" {
        show_main_window(window, false);
    } else {
        show_management_window(window, false);
    }

    let _ = window.emit(NAVIGATE_EVENT, route);
}

pub fn show_management_window(window: &WebviewWindow, center_on_show: bool) {
    let _ = window.set_size(Size::Logical(LogicalSize {
        width: MANAGEMENT_WINDOW_WIDTH,
        height: MANAGEMENT_WINDOW_HEIGHT,
    }));
    if center_on_show {
        let _ = window.center();
    }
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn hide_main_window(window: &WebviewWindow) {
    let _ = window.hide();
}

pub fn toggle_main_window(window: &WebviewWindow, center_on_show: bool) {
    if window.is_visible().unwrap_or(false) {
        hide_main_window(window);
    } else {
        open_route(window, "/", center_on_show);
    }
}
