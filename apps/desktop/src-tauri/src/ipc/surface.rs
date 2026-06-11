use litools_index::repository::AppRepository;
use tauri::{AppHandle, Manager, State, Webview};
use tauri_plugin_opener::OpenerExt;

use crate::{
    state::AppState,
    surface::{model::SurfaceMetadata, service},
    windowing::{labels, native},
};

#[tauri::command]
pub fn detach_route(
    route: String,
    webview: Webview,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<SurfaceMetadata, String> {
    service::detach_current_surface(&app_handle, &state, &webview, &route)
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
    service::open_view_route(&app_handle, &state, "/")
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

#[tauri::command]
pub fn reveal_in_file_manager(result_id: String, state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    let app_id = litools_core::app_provider::app_id_from_result_id(&result_id)
        .ok_or_else(|| format!("非应用结果：{result_id}"))?;

    let app_lock = state.app().lock().map_err(|error| error.to_string())?;
    let connection = app_lock.context().database.connection();
    let app = AppRepository::new(&connection)
        .find_app(app_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("应用未找到：{app_id}"))?;

    let app_path = std::path::Path::new(&app.path);
    app_handle.opener().reveal_item_in_dir(app_path)
        .map_err(|e| format!("无法定位文件：{e}"))
}
