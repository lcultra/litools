use tauri::{Manager, Size, Window, LogicalSize, window::WindowBuilder};

use crate::{
    state::AppState,
    view::ViewKind,
    windowing::labels::MAIN_WINDOW_LABEL,
};

use super::positioning::{center_window_on_show, position_launcher_on_show};
pub use litools_config::window::{
    DEFAULT_MAIN_WINDOW_HEIGHT, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH,
};

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
        .inner_size(DEFAULT_WINDOW_WIDTH, DEFAULT_MAIN_WINDOW_HEIGHT)
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

pub fn show_launcher_host(window: &Window, state: &AppState) {
    state.begin_programmatic_layout();
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
    state.begin_programmatic_layout();
    configure_space_behavior(window);
    let _ = set_panel_size(window);
    position_launcher_on_show(window, state);
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
        let scale = window.scale_factor().map_err(|error| error.to_string())?;
        let old_size = window
            .outer_size()
            .map_err(|error| error.to_string())?
            .to_logical::<f64>(scale);
        // resize 可能异步触发 Moved 事件；begin_programmatic_layout
        // 开启 100ms 窗口将其忽略，不写入位置记忆。
        if let Some(state) = app.try_state::<AppState>() {
            state.begin_programmatic_layout();
        }
        window
            .set_size(Size::Logical(LogicalSize { width: old_size.width, height }))
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub fn emit_focus_to_owned_launcher_surfaces(window: &Window) {
    for webview in window.webviews() {
        let Some(metadata) = window
            .try_state::<crate::state::AppState>()
            .and_then(|state| {
                state
                    .surfaces
                    .lock()
                    .ok()
                    .and_then(|r| r.metadata_for_webview_label(webview.label()))
            })
        else {
            continue;
        };

        if metadata.host_window_label == window.label() {
            crate::core::surface::events::emit_focus_search(&webview);
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
