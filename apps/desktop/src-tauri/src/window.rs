use std::path::PathBuf;

use tauri::{
    Emitter, LogicalPosition, LogicalSize, Manager, Size, Webview, WebviewUrl, Window,
    webview::WebviewBuilder,
    window::WindowBuilder,
};

use crate::state::{AppState, ManagedWindowKind, ManagedWindowLifecycle, ManagedWindowMetadata};

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const DETACHED_WINDOW_PREFIX: &str = "detached-management-";
pub const FOCUS_SEARCH_EVENT: &str = "focus-search";
pub const NAVIGATE_EVENT: &str = "navigate";
pub const SURFACE_METADATA_CHANGED_EVENT: &str = "surface-metadata-changed";

const MANAGEMENT_WINDOW_WIDTH: f64 = 820.0;
const MANAGEMENT_WINDOW_HEIGHT: f64 = 560.0;

pub fn main_window(app: &tauri::AppHandle) -> Option<Window> {
    app.get_window(MAIN_WINDOW_LABEL)
}

pub fn is_detached_window_label(label: &str) -> bool {
    label.starts_with(DETACHED_WINDOW_PREFIX)
}

pub fn validate_detachable_route(route: &str) -> Result<(), String> {
    if is_detachable_route(route) {
        Ok(())
    } else {
        Err(format!("route cannot be detached: {route}"))
    }
}

pub fn validate_surface_route(route: &str, owner_window_label: &str) -> Result<(), String> {
    match route {
        "/" if owner_window_label == MAIN_WINDOW_LABEL => Ok(()),
        "/settings" | "/plugins" | "/diagnostics" => Ok(()),
        "/" => Err("detached management surfaces cannot navigate to launcher".to_string()),
        _ => Err(format!("unknown route: {route}")),
    }
}

fn is_detachable_route(route: &str) -> bool {
    matches!(route, "/settings" | "/plugins" | "/diagnostics")
}

fn management_size() -> Size {
    Size::Logical(LogicalSize {
        width: MANAGEMENT_WINDOW_WIDTH,
        height: MANAGEMENT_WINDOW_HEIGHT,
    })
}

pub fn emit_surface_metadata_changed(app: &tauri::AppHandle, metadata: &ManagedWindowMetadata) {
    if let Some(webview) = app.get_webview(&metadata.webview_label) {
        let _ = webview.emit(SURFACE_METADATA_CHANGED_EVENT, metadata.clone());
    }
}

fn add_surface_webview(
    window: &Window,
    metadata: &ManagedWindowMetadata,
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
    webview.set_auto_resize(true).map_err(|error| error.to_string())?;
    Ok(webview)
}

pub fn create_main_window(app: &tauri::AppHandle, state: &AppState) -> Result<Window, String> {
    if let Some(window) = main_window(app) {
        if window.webviews().is_empty() {
            let metadata = state.register_main_surface()?;
            add_surface_webview(&window, &metadata, "/")?;
        }
        return Ok(window);
    }

    let window = WindowBuilder::new(app, MAIN_WINDOW_LABEL)
        .title("litools")
        .inner_size(MANAGEMENT_WINDOW_WIDTH, MANAGEMENT_WINDOW_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .build()
        .map_err(|error| error.to_string())?;
    let metadata = state.register_main_surface()?;
    add_surface_webview(&window, &metadata, "/")?;
    Ok(window)
}

pub fn ensure_main_surface(app: &tauri::AppHandle, state: &AppState) -> Result<Window, String> {
    let window = main_window(app).ok_or_else(|| "main window not found".to_string())?;
    let has_main_webview = window.webviews().iter().any(|webview| {
        state
            .window_metadata_for_webview_label(webview.label())
            .is_some_and(|metadata| {
                metadata.owner_window_label == MAIN_WINDOW_LABEL
                    && metadata.kind == ManagedWindowKind::Main
                    && metadata.lifecycle != ManagedWindowLifecycle::Destroyed
            })
    });

    if !has_main_webview {
        let metadata = state.register_main_surface()?;
        add_surface_webview(&window, &metadata, "/")?;
    }
    Ok(window)
}

fn create_replacement_main_surface(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<(Window, Webview), String> {
    let window = main_window(app).ok_or_else(|| "main window not found".to_string())?;
    let metadata = state.register_main_surface()?;
    let webview = add_surface_webview(&window, &metadata, "/")?;
    Ok((window, webview))
}

pub fn show_main_window(window: &Window, center_on_show: bool) {
    if center_on_show {
        let _ = window.center();
    }
    let _ = window.show();
    let _ = window.set_focus();
    for webview in window.webviews() {
        let Some(metadata) = window
            .try_state::<AppState>()
            .and_then(|state| state.window_metadata_for_webview_label(webview.label()))
        else {
            continue;
        };

        if metadata.owner_window_label == window.label() {
            let _ = webview.emit(FOCUS_SEARCH_EVENT, ());
        }
    }
}

pub fn show_management_window(window: &Window, center_on_show: bool) {
    let _ = window.set_size(management_size());
    if center_on_show {
        let _ = window.center();
    }
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn open_route(window: &Window, route: &str, center_on_show: bool) {
    if route == "/" {
        show_main_window(window, center_on_show);
    } else {
        show_management_window(window, center_on_show);
    }

    for webview in window.webviews() {
        let Some(metadata) = window
            .try_state::<AppState>()
            .and_then(|state| state.window_metadata_for_webview_label(webview.label()))
        else {
            continue;
        };

        if metadata.owner_window_label == window.label() {
            let _ = webview.emit(NAVIGATE_EVENT, route);
        }
    }
}

fn claim_or_create_detached_window(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<Window, String> {
    let label = state.next_detached_window_label()?;
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

pub fn window_by_id_or_label(
    app: &tauri::AppHandle,
    state: &AppState,
    target: &str,
) -> Option<Window> {
    if target == MAIN_WINDOW_LABEL {
        return main_window(app);
    }

    state
        .window_metadata(target)
        .and_then(|metadata| app.get_window(&metadata.owner_window_label))
        .or_else(|| app.get_window(target))
}

pub fn detach_route(
    app: &tauri::AppHandle,
    state: &AppState,
    webview: &Webview,
    route: &str,
    center_on_show: bool,
) -> Result<ManagedWindowMetadata, String> {
    validate_detachable_route(route)?;

    let current_metadata = state
        .window_metadata_for_webview_label(webview.label())
        .ok_or_else(|| format!("surface metadata not found: {}", webview.label()))?;
    if current_metadata.lifecycle == ManagedWindowLifecycle::Destroyed {
        return Err(format!("surface has been destroyed: {}", webview.label()));
    }
    if current_metadata.owner_window_label != MAIN_WINDOW_LABEL
        || current_metadata.kind == ManagedWindowKind::DetachedManagement
    {
        return Err(format!("surface is already detached: {}", webview.label()));
    }

    let source_window_label = current_metadata.owner_window_label.clone();
    let detached_window = claim_or_create_detached_window(app, state)?;
    let detached_window_label = detached_window.label().to_string();

    webview
        .reparent(&detached_window)
        .map_err(|error| error.to_string())?;
    webview
        .set_position(LogicalPosition::new(0, 0))
        .map_err(|error| error.to_string())?;
    webview
        .set_size(detached_window.inner_size().map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    webview.set_auto_resize(true).map_err(|error| error.to_string())?;

    let metadata = state
        .move_surface_to_window(
            webview.label(),
            detached_window_label,
            ManagedWindowKind::DetachedManagement,
            Some(route.to_string()),
        )
        .ok_or_else(|| format!("surface metadata not found after reparent: {}", webview.label()))?;

    show_management_window(&detached_window, center_on_show);
    emit_surface_metadata_changed(app, &metadata);

    if source_window_label == MAIN_WINDOW_LABEL {
        match create_replacement_main_surface(app, state) {
            Ok((main_window, main_webview)) => {
                show_main_window(&main_window, center_on_show);
                let _ = main_webview.emit(FOCUS_SEARCH_EVENT, ());
            }
            Err(error) => {
                let _ = ensure_main_surface(app, state);
                return Err(format!("detached surface moved, but main surface recovery failed: {error}"));
            }
        }
    }

    Ok(metadata)
}

pub fn hide_window(window: &Window) {
    let _ = window.hide();
}

pub fn hide_managed_window(
    app: &tauri::AppHandle,
    state: &AppState,
    target: &str,
) -> Result<(), String> {
    let metadata = state
        .window_metadata(target)
        .ok_or_else(|| format!("window metadata not found: {target}"))?;
    let window = app
        .get_window(&metadata.owner_window_label)
        .ok_or_else(|| format!("window not found: {}", metadata.owner_window_label))?;
    hide_window(&window);
    if let Some(metadata) = state.mark_window_lifecycle(&metadata.webview_label, ManagedWindowLifecycle::Hidden) {
        emit_surface_metadata_changed(app, &metadata);
    }
    Ok(())
}

pub fn hide_main_window(window: &Window) {
    hide_window(window);
}

pub fn focus_window(window: &Window) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

pub fn destroy_managed_window(
    app: &tauri::AppHandle,
    state: &AppState,
    target: &str,
) -> Result<(), String> {
    let metadata = state
        .window_metadata(target)
        .ok_or_else(|| format!("window metadata not found: {target}"))?;

    if metadata.owner_window_label == MAIN_WINDOW_LABEL {
        return Err("main window cannot be destroyed through managed window API".to_string());
    }

    if let Some(webview) = app.get_webview(&metadata.webview_label) {
        if let Some(metadata) = state.mark_window_lifecycle(&metadata.webview_label, ManagedWindowLifecycle::Destroyed) {
            emit_surface_metadata_changed(app, &metadata);
        }
        webview.close().map_err(|error| error.to_string())?;
    }
    if let Some(window) = app.get_window(&metadata.owner_window_label) {
        window.destroy().map_err(|error| error.to_string())?;
    }
    state.remove_window(&metadata.webview_label);
    Ok(())
}

pub fn toggle_main_window(window: &Window, center_on_show: bool) {
    if window.is_visible().unwrap_or(false) {
        hide_main_window(window);
    } else {
        open_route(window, "/", center_on_show);
    }
}
