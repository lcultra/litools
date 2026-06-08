use std::path::PathBuf;

use tauri::{
    LogicalPosition, LogicalSize, Manager, Size, Webview, WebviewUrl, Window,
    webview::WebviewBuilder, window::WindowBuilder,
};

use crate::{
    surface::{events, model::SurfaceMetadata},
    view::model::ViewKind,
    windowing::labels::MAIN_WINDOW_LABEL,
};

const MANAGEMENT_WINDOW_WIDTH: f64 = 820.0;
const MANAGEMENT_WINDOW_HEIGHT: f64 = 560.0;

pub fn main_window(app: &tauri::AppHandle) -> Option<Window> {
    app.get_window(MAIN_WINDOW_LABEL)
}

pub fn create_main_host(app: &tauri::AppHandle) -> Result<Window, String> {
    if let Some(window) = main_window(app) {
        return Ok(window);
    }

    WindowBuilder::new(app, MAIN_WINDOW_LABEL)
        .title("litools")
        .inner_size(MANAGEMENT_WINDOW_WIDTH, MANAGEMENT_WINDOW_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .build()
        .map_err(|error| error.to_string())
}

pub fn create_detached_panel_host(app: &tauri::AppHandle, label: String) -> Result<Window, String> {
    if let Some(window) = app.get_window(&label) {
        return Ok(window);
    }

    WindowBuilder::new(app, label)
        .title("litools")
        .inner_size(MANAGEMENT_WINDOW_WIDTH, MANAGEMENT_WINDOW_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .visible(false)
        .build()
        .map_err(|error| error.to_string())
}

pub fn add_surface_webview(
    window: &Window,
    metadata: &SurfaceMetadata,
    route: &str,
) -> Result<Webview, String> {
    let url = format!("index.html#{route}");
    let size = window.inner_size().map_err(|error| error.to_string())?;
    let webview = window
        .add_child(
            WebviewBuilder::new(
                metadata.webview_label.clone(),
                WebviewUrl::App(PathBuf::from(url)),
            )
            .transparent(true)
            .auto_resize(),
            LogicalPosition::new(0, 0),
            size,
        )
        .map_err(|error| error.to_string())?;
    webview
        .set_auto_resize(true)
        .map_err(|error| error.to_string())?;
    Ok(webview)
}

pub fn show_launcher_host(window: &Window, center_on_show: bool) {
    if center_on_show {
        let _ = window.center();
    }
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn show_panel_host(window: &Window, center_on_show: bool) {
    let _ = set_panel_size(window);
    if center_on_show {
        let _ = window.center();
    }
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn show_host_for_view(window: &Window, view_kind: &ViewKind, center_on_show: bool) {
    match view_kind {
        ViewKind::Launcher => show_launcher_host(window, center_on_show),
        ViewKind::Panel | ViewKind::Runtime => show_panel_host(window, center_on_show),
    }
}

pub fn hide_window(window: &Window) {
    let _ = window.hide();
}

pub fn focus_window(window: &Window) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

pub fn set_panel_size(window: &Window) -> Result<(), String> {
    window
        .set_size(management_size())
        .map_err(|error| error.to_string())
}

pub fn resize_main_window_height(app: &tauri::AppHandle, height: f64) -> Result<(), String> {
    if let Some(window) = main_window(app) {
        let width = window
            .outer_size()
            .map_err(|error| error.to_string())?
            .to_logical::<f64>(window.scale_factor().map_err(|error| error.to_string())?)
            .width;
        window
            .set_size(Size::Logical(LogicalSize { width, height }))
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub fn emit_focus_to_owned_launcher_surfaces(window: &Window) {
    for webview in window.webviews() {
        let Some(metadata) = window
            .try_state::<crate::state::AppState>()
            .and_then(|state| state.surface_metadata_for_webview_label(webview.label()))
        else {
            continue;
        };

        if metadata.host_window_label == window.label() {
            events::emit_focus_search(&webview);
        }
    }
}

fn management_size() -> Size {
    Size::Logical(LogicalSize {
        width: MANAGEMENT_WINDOW_WIDTH,
        height: MANAGEMENT_WINDOW_HEIGHT,
    })
}
