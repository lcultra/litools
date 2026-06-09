use tauri::{AppHandle, Manager, State, Webview};

use crate::{
    plugin_runtime,
    state::AppState,
    surface::{model::SurfaceMetadata, service},
    view::registry,
    windowing::{labels, native},
};

#[tauri::command]
pub fn detach_route(
    route: String,
    webview: Webview,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<SurfaceMetadata, String> {
    service::detach_current_surface(
        &app_handle,
        &state,
        &webview,
        &route,
        state.center_on_show(),
    )
}

#[tauri::command]
pub fn update_surface_route(
    route: String,
    webview: Webview,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<SurfaceMetadata, String> {
    service::update_surface_route(&app_handle, &state, webview.label(), &route)
}

#[tauri::command]
pub fn open_route(
    route: String,
    target: Option<String>,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    if target.as_deref() == Some("detached") {
        return Err("open_route with detached target does not move the current page; use detach_route from the current surface".to_string());
    }

    if target.as_deref() == Some("runtime") {
        let Some((plugin_id, command_id)) = registry::plugin_route_parts(&route) else {
            return Err(format!(
                "runtime target requires a plugin route: {route}"
            ));
        };
        plugin_runtime::service::dock_plugin_runtime(
            &app_handle,
            &state,
            plugin_id,
            command_id,
            state.center_on_show(),
        )?;
        return Ok(());
    }

    service::open_view_route(&app_handle, &state, &route, state.center_on_show())
}

#[tauri::command]
pub fn list_windows(state: State<'_, AppState>) -> Result<Vec<SurfaceMetadata>, String> {
    Ok(service::list_surfaces(&state))
}

#[tauri::command]
pub fn get_current_window_metadata(
    webview: Webview,
    state: State<'_, AppState>,
) -> Result<Option<SurfaceMetadata>, String> {
    Ok(service::current_surface_metadata(&state, webview.label()))
}

#[tauri::command]
pub fn hide_window(
    target: Option<String>,
    webview: Webview,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let target = target.unwrap_or_else(|| webview.label().to_string());

    if target == labels::MAIN_WINDOW_LABEL {
        if let Some(window) = native::main_window(&app_handle) {
            native::hide_window(&window);
        }
        return Ok(());
    }

    service::hide_surface(&app_handle, &state, &target)
}

#[tauri::command]
pub fn focus_window(
    target: Option<String>,
    webview: Webview,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    service::focus_surface_or_host(&app_handle, &state, target.as_deref(), webview.window())
}

#[tauri::command]
pub fn destroy_window(
    target: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    service::destroy_surface(&app_handle, &state, &target)
}

#[tauri::command]
pub fn start_window_dragging(webview: Webview) -> Result<(), String> {
    webview
        .window()
        .start_dragging()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn hide_main_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = native::main_window(&app_handle) {
        native::hide_window(&window);
    }

    Ok(())
}

#[tauri::command]
pub fn show_main_window(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<AppState>();
    service::open_view_route(&app_handle, &state, "/", state.center_on_show())
}

#[tauri::command]
pub fn focus_main_window(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<AppState>();
    let window = service::ensure_main_launcher_surface(&app_handle, &state)?;
    window.set_focus().map_err(|error| error.to_string())?;
    native::emit_focus_to_owned_launcher_surfaces(&window);

    Ok(())
}

#[tauri::command]
pub fn resize_main_window_height(height: f64, app_handle: AppHandle) -> Result<(), String> {
    native::resize_main_window_height(&app_handle, height)
}
