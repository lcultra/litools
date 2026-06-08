use litools_core::ReloadIndexSummary;
use litools_settings::AppSettings;
use serde::Serialize;
use tauri::{Manager, State};

use crate::{
    app_watcher::AppWatcherStatus,
    icon_cache::{IconCacheSummary, icon_cache_summary},
    index_refresh::{IndexRefreshTrigger, IndexStatus, request_index_refresh},
    state::AppState,
};

#[tauri::command]
pub fn reload_index(app_handle: tauri::AppHandle) -> Result<IndexStatus, String> {
    request_index_refresh(&app_handle, IndexRefreshTrigger::Manual);
    Ok(app_handle.state::<AppState>().index_status())
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
    app_count: usize,
    index_status: IndexStatus,
    last_persisted_index_status: Option<ReloadIndexSummary>,
    app_watcher: AppWatcherStatus,
    icon_cache: IconCacheSummary,
    recent_usage_count: usize,
    recent_usage: Vec<UsageEventResponse>,
    settings: AppSettings,
    shortcut: crate::state::ShortcutStatus,
}

#[tauri::command]
pub fn get_diagnostics(state: State<'_, AppState>) -> Result<DiagnosticsResponse, String> {
    get_diagnostics_inner(&state)
}

pub fn get_diagnostics_inner(state: &AppState) -> Result<DiagnosticsResponse, String> {
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
        plugin_count: app.context().plugins.len(),
        command_count: app.command_count().map_err(|error| error.to_string())?,
        app_count: app.app_count().map_err(|error| error.to_string())?,
        index_status: state.index_status(),
        last_persisted_index_status: app.index_status().map_err(|error| error.to_string())?,
        app_watcher: state.app_watcher_status(),
        icon_cache: icon_cache_summary(state.data_dir()),
        recent_usage_count,
        recent_usage,
        settings: app.settings().clone(),
        shortcut: state.shortcut_status(),
    })
}
