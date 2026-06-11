use serde_json::Value;
use tauri::{AppHandle, Manager, State, Webview};

use crate::{
    core::plugins::{
        dispatch::route_plugin_view_call,
        runtime::{
            model::{PluginRuntimeError, PluginRuntimeInfo},
            service,
        },
    },
    state::AppState,
};

#[tauri::command]
pub async fn open_plugin_view(
    plugin_id: String,
    command_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<PluginRuntimeInfo, String> {
    service::dock_plugin_runtime(&app_handle, &state, &plugin_id, &command_id)
        .map(|context| service::build_runtime_info(&state, &context))
}

#[tauri::command]
pub fn hide_plugin_view(
    plugin_id: String,
    command_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<PluginRuntimeInfo, String> {
    service::hide_docked_plugin_runtime(&app_handle, &state, &plugin_id, &command_id)
        .map(|context| service::build_runtime_info(&state, &context))
}

#[tauri::command]
pub async fn detach_plugin_view(
    plugin_id: String,
    command_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<PluginRuntimeInfo, String> {
    service::detach_plugin_runtime(&app_handle, &state, &plugin_id, &command_id)
        .map(|context| service::build_runtime_info(&state, &context))
}

#[tauri::command]
pub fn close_plugin_view(
    plugin_id: String,
    command_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    service::close_plugin_runtime_by_plugin_command(&app_handle, &state, &plugin_id, &command_id)
}

#[tauri::command]
pub fn close_plugin_view_by_id(
    runtime_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    service::close_runtime(&app_handle, &state, &runtime_id)
}

#[tauri::command]
pub fn get_plugin_view_info(
    runtime_id: String,
    state: State<'_, AppState>,
) -> Result<PluginRuntimeInfo, String> {
    service::runtime_info(&state, &runtime_id)
}

/// 打开插件 webview 的开发者工具（仅开发模式）
#[tauri::command]
pub fn open_plugin_devtools(
    runtime_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let context = state
        .plugin_runtimes
        .lock()
        .ok()
        .and_then(|r| r.runtime(&runtime_id))
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    let webview_label = {
        let surfaces = state.surfaces.lock().ok()
            .ok_or_else(|| "failed to lock surfaces".to_string())?;
        surfaces.webview_label(&context.surface_id)
            .ok_or_else(|| format!("surface not found: {}", context.surface_id))?
            .to_string()
    };
    let webview = app_handle
        .get_webview(&webview_label)
        .ok_or_else(|| format!("webview not found: {}", webview_label))?;
    webview.open_devtools();
    Ok(())
}

/// 公开入口，供 SDK 命令复用 dispatch 逻辑。
pub fn route_plugin_view_call_inner(
    method: &str,
    params: Value,
    webview: &Webview,
    state: &AppState,
    app_handle: &AppHandle,
) -> Result<Value, PluginRuntimeError> {
    route_plugin_view_call(method, params, webview, state, app_handle)
}
