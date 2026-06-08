use litools_core::{
    BuiltinCommandEffect, CommandExecution, LauncherPanelResponse, ReloadIndexSummary,
};
use litools_search::SearchResult;
use litools_settings::AppSettings;
use serde::Serialize;
use tauri::{AppHandle, Emitter, LogicalSize, Manager, Size, State, Webview};

use crate::{
    app_watcher::AppWatcherStatus,
    icon_cache::{IconCacheSummary, icon_cache_summary},
    index_refresh::{IndexRefreshTrigger, IndexStatus, request_index_refresh},
    shortcut,
    state::{AppState, ManagedWindowLifecycle, ManagedWindowMetadata},
    window,
};

#[tauri::command]
pub fn search(query: String, state: State<'_, AppState>) -> Result<Vec<SearchResult>, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    Ok(app.search(query))
}

#[tauri::command]
pub fn launcher_panel(
    query: String,
    state: State<'_, AppState>,
) -> Result<LauncherPanelResponse, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    app.launcher_panel(query).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn pin_result(result_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    app.pin_result(result_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn unpin_result(result_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    app.unpin_result(result_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn reorder_pinned_results(
    result_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    app.reorder_pinned_results(result_ids)
        .map_err(|error| error.to_string())
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
            let window = window::ensure_main_surface(&app_handle, &state)?;
            window::open_route(&window, "/settings", state.center_on_show());
        }
        BuiltinCommandEffect::OpenLogs => {
            let window = window::ensure_main_surface(&app_handle, &state)?;
            window::open_route(&window, "/diagnostics", state.center_on_show());
        }
        _ => {}
    }

    Ok(execution)
}

#[tauri::command]
pub fn detach_route(
    route: String,
    webview: Webview,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<ManagedWindowMetadata, String> {
    window::detach_route(&app_handle, &state, &webview, &route, state.center_on_show())
}

#[tauri::command]
pub fn update_surface_route(
    route: String,
    webview: Webview,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<ManagedWindowMetadata, String> {
    let current_metadata = state
        .window_metadata_for_webview_label(webview.label())
        .ok_or_else(|| format!("surface metadata not found: {}", webview.label()))?;
    window::validate_surface_route(&route, &current_metadata.owner_window_label)?;
    if current_metadata.route.as_deref() == Some(route.as_str()) {
        return Ok(current_metadata);
    }

    let metadata = state
        .mark_window_route(webview.label(), route)
        .ok_or_else(|| format!("surface metadata not found: {}", webview.label()))?;
    window::emit_surface_metadata_changed(&app_handle, &metadata);
    Ok(metadata)
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

    let window = window::ensure_main_surface(&app_handle, &state)?;
    window::open_route(&window, &route, state.center_on_show());

    Ok(())
}

#[tauri::command]
pub fn list_windows(state: State<'_, AppState>) -> Result<Vec<ManagedWindowMetadata>, String> {
    Ok(state.list_windows())
}

#[tauri::command]
pub fn get_current_window_metadata(
    webview: Webview,
    state: State<'_, AppState>,
) -> Result<Option<ManagedWindowMetadata>, String> {
    Ok(state.window_metadata_for_webview_label(webview.label()))
}

#[tauri::command]
pub fn hide_window(
    target: Option<String>,
    webview: Webview,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let target = target.unwrap_or_else(|| webview.label().to_string());

    if target == window::MAIN_WINDOW_LABEL {
        if let Some(window) = window::main_window(&app_handle) {
            window::hide_window(&window);
        }
        return Ok(());
    }

    window::hide_managed_window(&app_handle, &state, &target)
}

#[tauri::command]
pub fn focus_window(
    target: Option<String>,
    webview: Webview,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let target_window = match target.as_deref() {
        Some("main") => window::main_window(&app_handle),
        Some(target) => window::window_by_id_or_label(&app_handle, &state, target),
        None => Some(webview.window()),
    };

    if let Some(target_window) = target_window {
        window::focus_window(&target_window)?;
        if let Some(metadata) = state.mark_window_lifecycle(target_window.label(), ManagedWindowLifecycle::Active) {
            window::emit_surface_metadata_changed(&app_handle, &metadata);
        }
        if let Some(metadata) = state.mark_window_focused(target_window.label(), true) {
            window::emit_surface_metadata_changed(&app_handle, &metadata);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn destroy_window(
    target: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    window::destroy_managed_window(&app_handle, &state, &target)
}

#[tauri::command]
pub fn start_window_dragging(webview: Webview) -> Result<(), String> {
    webview.window().start_dragging().map_err(|error| error.to_string())
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
    let state = app_handle.state::<AppState>();
    let window = window::ensure_main_surface(&app_handle, &state)?;
    window::show_main_window(&window, state.center_on_show());

    Ok(())
}

#[tauri::command]
pub fn open_settings(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<AppState>();
    let window = window::ensure_main_surface(&app_handle, &state)?;
    window::open_route(&window, "/settings", state.center_on_show());

    Ok(())
}

#[tauri::command]
pub fn focus_main_window(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<AppState>();
    let window = window::ensure_main_surface(&app_handle, &state)?;
    window.set_focus().map_err(|error| error.to_string())?;
    for webview in window.webviews() {
        webview
            .emit(window::FOCUS_SEARCH_EVENT, ())
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn start_dragging(webview: Webview) -> Result<(), String> {
    start_window_dragging(webview)
}

#[tauri::command]
pub fn resize_main_window_height(height: f64, app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = window::main_window(&app_handle) {
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

#[tauri::command]
pub fn reload_index(app_handle: AppHandle) -> Result<IndexStatus, String> {
    request_index_refresh(&app_handle, IndexRefreshTrigger::Manual);
    Ok(app_handle.state::<AppState>().index_status())
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
        app.update_settings(settings)
            .map_err(|error| error.to_string())?
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
