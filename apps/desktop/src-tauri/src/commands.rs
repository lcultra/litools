use litools_search::SearchResult;
use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn search(query: String, state: State<'_, AppState>) -> Result<Vec<SearchResult>, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    Ok(app.search(query))
}

#[tauri::command]
pub fn execute_result(result_id: String, action_id: String) -> Result<String, String> {
    Ok(format!(
        "queued action `{action_id}` for result `{result_id}`"
    ))
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
pub struct DiagnosticsResponse {
    app_version: String,
    plugin_count: usize,
}

#[tauri::command]
pub fn get_diagnostics(state: State<'_, AppState>) -> Result<DiagnosticsResponse, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    Ok(DiagnosticsResponse {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        plugin_count: app.context().plugins.installed_plugins().len(),
    })
}
