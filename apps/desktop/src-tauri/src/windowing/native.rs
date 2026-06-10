use std::path::PathBuf;

use tauri::{
    LogicalPosition, LogicalSize, Manager, Position, Size, Webview, WebviewUrl, Window,
    webview::WebviewBuilder, window::WindowBuilder,
};

use crate::{
    plugin_runtime::model::PluginRuntimeBounds,
    state::AppState,
    surface::{events, model::SurfaceMetadata},
    view::model::ViewKind,
    windowing::labels::MAIN_WINDOW_LABEL,
};

use super::positioning::{
    center_main_window_on_show, center_window_on_show, position_launcher_on_show,
};
pub use litools_config::window::{
    CHROME_INSET, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, TITLEBAR_HEIGHT,
};

/// 1px WindowFrame p-px + 1px Panel border

pub fn main_window(app: &tauri::AppHandle) -> Option<Window> {
    app.get_window(MAIN_WINDOW_LABEL)
}

pub fn create_main_host(app: &tauri::AppHandle) -> Result<Window, String> {
    if let Some(window) = main_window(app) {
        configure_space_behavior(&window);
        return Ok(window);
    }

    let window = WindowBuilder::new(app, MAIN_WINDOW_LABEL)
        .title("litools")
        .inner_size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .visible_on_all_workspaces(true)
        .build()
        .map_err(|error| error.to_string())?;
    configure_space_behavior(&window);
    Ok(window)
}

pub fn create_detached_panel_host(app: &tauri::AppHandle, label: String) -> Result<Window, String> {
    if let Some(window) = app.get_window(&label) {
        configure_space_behavior(&window);
        return Ok(window);
    }

    let window = WindowBuilder::new(app, label)
        .title("litools")
        .inner_size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .visible(false)
        .visible_on_all_workspaces(true)
        .build()
        .map_err(|error| error.to_string())?;
    configure_space_behavior(&window);
    Ok(window)
}

pub fn create_plugin_runtime_detached_host(
    app: &tauri::AppHandle,
    window_label: String,
    title: &str,
) -> Result<Window, String> {
    if let Some(window) = app.get_window(&window_label) {
        configure_space_behavior(&window);
        center_window_on_show(&window);
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(window);
    }

    let window = WindowBuilder::new(app, window_label)
        .title(title)
        .inner_size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .visible(false)
        .visible_on_all_workspaces(true)
        .build()
        .map_err(|error| error.to_string())?;
    configure_space_behavior(&window);
    center_window_on_show(&window);
    Ok(window)
}

pub fn add_plugin_runtime_webview(
    window: &Window,
    webview_label: String,
    entry_url: &str,
    initialization_script: String,
) -> Result<(Webview, PluginRuntimeBounds), String> {
    let url = tauri::Url::parse(entry_url).map_err(|error| error.to_string())?;
    let bounds = plugin_runtime_content_bounds(window)?;
    let webview = window
        .add_child(
            WebviewBuilder::new(webview_label, WebviewUrl::CustomProtocol(url))
                .initialization_script(initialization_script)
                .transparent(true),
            LogicalPosition::new(bounds.x, bounds.y),
            Size::Logical(LogicalSize {
                width: bounds.width,
                height: bounds.height,
            }),
        )
        .map_err(|error| error.to_string())?;
    webview
        .set_auto_resize(false)
        .map_err(|error| error.to_string())?;
    Ok((webview, bounds))
}

pub fn set_plugin_runtime_content_bounds(
    window: &Window,
    webview: &Webview,
) -> Result<PluginRuntimeBounds, String> {
    let bounds = plugin_runtime_content_bounds(window)?;
    webview
        .set_position(Position::Logical(LogicalPosition::new(bounds.x, bounds.y)))
        .map_err(|error| error.to_string())?;
    webview
        .set_size(Size::Logical(LogicalSize {
            width: bounds.width,
            height: bounds.height,
        }))
        .map_err(|error| error.to_string())?;
    Ok(bounds)
}

pub fn hide_plugin_runtime_webview(webview: &Webview) -> Result<(), String> {
    webview.hide().map_err(|error| error.to_string())
}

pub fn show_plugin_runtime_webview(webview: &Webview) -> Result<(), String> {
    webview.show().map_err(|error| error.to_string())
}

pub fn plugin_runtime_content_bounds(window: &Window) -> Result<PluginRuntimeBounds, String> {
    let size = window_inner_logical_size(window)?;
    Ok(PluginRuntimeBounds {
        x: CHROME_INSET,
        y: TITLEBAR_HEIGHT + CHROME_INSET,
        width: (size.width - CHROME_INSET * 2.0).max(0.0),
        height: (size.height - TITLEBAR_HEIGHT - CHROME_INSET * 2.0).max(0.0),
    })
}

fn window_inner_logical_size(window: &Window) -> Result<LogicalSize<f64>, String> {
    let size = window.inner_size().map_err(|error| error.to_string())?;
    let scale_factor = window.scale_factor().map_err(|error| error.to_string())?;
    Ok(size.to_logical::<f64>(scale_factor))
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

pub fn show_launcher_host(window: &Window, state: &AppState) {
    configure_space_behavior(window);
    position_launcher_on_show(window, state);
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn show_panel_host(window: &Window) {
    configure_space_behavior(window);
    let _ = set_panel_size(window);
    center_window_on_show(window);
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn show_main_panel_host(window: &Window, state: &AppState) {
    configure_space_behavior(window);
    let _ = set_panel_size(window);
    center_main_window_on_show(window, state);
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn show_host_for_view(window: &Window, state: &AppState, view_kind: &ViewKind) {
    match view_kind {
        ViewKind::Launcher => show_launcher_host(window, state),
        ViewKind::Plugin if window.label() == MAIN_WINDOW_LABEL => {
            show_main_panel_host(window, state)
        }
        ViewKind::Plugin => show_panel_host(window),
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
        width: DEFAULT_WINDOW_WIDTH,
        height: DEFAULT_WINDOW_HEIGHT,
    })
}

fn configure_space_behavior(window: &Window) {
    let _ = window.set_visible_on_all_workspaces(true);
}
