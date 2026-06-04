use litools_core::{BuiltinCommandEffect, CommandExecution};
use litools_search::SearchResult;
use litools_settings::AppSettings;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::{shortcut, state::AppState, window};

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
    let execution = {
        let mut app = state.app().lock().map_err(|error| error.to_string())?;
        app.execute_result(result_id, action_id)
            .map_err(|error| error.to_string())?
    };

    match execution.effect {
        BuiltinCommandEffect::QuitApp => app_handle.exit(0),
        BuiltinCommandEffect::OpenSettings => {
            if let Some(window) = window::main_window(&app_handle) {
                window::show_view(&window, "settings", state.center_on_show());
            }
        }
        BuiltinCommandEffect::OpenLogs => {
            if let Some(window) = window::main_window(&app_handle) {
                window::show_view(&window, "diagnostics", state.center_on_show());
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
        window::show_main_window(&window, app_handle.state::<AppState>().center_on_show());
    }

    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    Ok(app.settings().clone())
}

#[tauri::command]
pub fn update_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<AppSettings, String> {
    let updated_settings = {
        let mut app = state.app().lock().map_err(|error| error.to_string())?;
        app.update_settings(settings).map_err(|error| error.to_string())?
    };

    shortcut::register_global_shortcut(&app_handle, &updated_settings.palette.global_hotkey);
    Ok(updated_settings)
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
    app_data_dir: String,
    platform: String,
    plugin_count: usize,
    command_count: usize,
    recent_usage_count: usize,
    recent_usage: Vec<UsageEventResponse>,
    settings: AppSettings,
    shortcut: crate::state::ShortcutStatus,
}

#[tauri::command]
pub fn get_diagnostics(state: State<'_, AppState>) -> Result<DiagnosticsResponse, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    let recent_usage: Vec<UsageEventResponse> = app
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

    let recent_usage_count = app.usage_event_count().map_err(|error| error.to_string())?;

    Ok(DiagnosticsResponse {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        app_data_dir: state.data_dir().display().to_string(),
        platform: std::env::consts::OS.to_string(),
        plugin_count: app.context().plugins.installed_plugins().len(),
        command_count: app.command_count().map_err(|error| error.to_string())?,
        recent_usage_count,
        recent_usage,
        settings: app.settings().clone(),
        shortcut: state.shortcut_status(),
    })
}
