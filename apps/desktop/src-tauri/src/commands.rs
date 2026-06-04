use litools_core::{BuiltinCommandEffect, CommandExecution};
use litools_search::SearchResult;
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::{state::AppState, window};

#[tauri::command]
pub fn search(query: String, state: State<'_, AppState>) -> Result<Vec<SearchResult>, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    Ok(app.search(query))
}

#[tauri::command]
pub fn execute_result(
    result_id: String,
    action_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<CommandExecution, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    let execution = app
        .execute_result(result_id, action_id)
        .map_err(|error| error.to_string())?;

    match execution.effect {
        BuiltinCommandEffect::QuitApp => app_handle.exit(0),
        BuiltinCommandEffect::OpenSettings => {
            if let Some(window) = window::main_window(&app_handle) {
                window::show_view(&window, "settings");
            }
        }
        BuiltinCommandEffect::OpenLogs => {
            if let Some(window) = window::main_window(&app_handle) {
                window::show_view(&window, "diagnostics");
            }
        }
        _ => {}
    }

    Ok(execution)
}

#[tauri::command]
pub fn hide_main_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = window::main_window(&app_handle) {
        window::hide_main_window(&window);
    }

    Ok(())
}

#[tauri::command]
pub fn show_main_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = window::main_window(&app_handle) {
        window::show_main_window(&window);
    }

    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    serde_json::to_value(app.context().settings.get()).map_err(|error| error.to_string())
}

#[derive(Serialize)]
pub struct PluginSummary {
    id: String,
    name: String,
    version: String,
    enabled: bool,
}

#[tauri::command]
pub fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginSummary>, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    Ok(app
        .context()
        .plugins
        .installed_plugins()
        .iter()
        .map(|plugin| PluginSummary {
            id: plugin.manifest.id.clone(),
            name: plugin.manifest.name.clone(),
            version: plugin.manifest.version.clone(),
            enabled: plugin.enabled,
        })
        .collect())
}

#[derive(Serialize)]
pub struct UsageEventResponse {
    target_type: String,
    target_id: String,
    query: Option<String>,
    selected_at: String,
}

#[derive(Serialize)]
pub struct DiagnosticsResponse {
    app_version: String,
    plugin_count: usize,
    recent_usage: Vec<UsageEventResponse>,
}

#[tauri::command]
pub fn get_diagnostics(state: State<'_, AppState>) -> Result<DiagnosticsResponse, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    let recent_usage = app
        .recent_usage_events(10)
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|event| UsageEventResponse {
            target_type: event.target_type,
            target_id: event.target_id,
            query: event.query,
            selected_at: event.selected_at,
        })
        .collect();

    Ok(DiagnosticsResponse {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        plugin_count: app.context().plugins.installed_plugins().len(),
        recent_usage,
    })
}
