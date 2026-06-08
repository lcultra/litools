use tauri::{LogicalPosition, Webview, Window};

pub fn reparent_surface_webview(webview: &Webview, target_window: &Window) -> Result<(), String> {
    reparent_webview_to_window(webview, target_window)?;
    webview
        .set_position(LogicalPosition::new(0, 0))
        .map_err(|error| error.to_string())?;
    webview
        .set_size(
            target_window
                .inner_size()
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    webview
        .set_auto_resize(true)
        .map_err(|error| error.to_string())
}

pub fn reparent_webview_to_window(webview: &Webview, target_window: &Window) -> Result<(), String> {
    webview
        .reparent(target_window)
        .map_err(|error| error.to_string())
}
