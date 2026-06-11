use litools_core::{CommandEffect, CommandExecution, LauncherPanelResponse};
use litools_search::SearchResult;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::state::AppState;

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
    app.launcher_panel(query)
        .map_err(|error| error.to_error_string())
}

#[tauri::command]
pub fn pin_result(result_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    app.pin_result(result_id)
        .map_err(|error| error.to_error_string())
}

#[tauri::command]
pub fn unpin_result(result_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    app.unpin_result(result_id)
        .map_err(|error| error.to_error_string())
}

#[tauri::command]
pub fn reorder_pinned_results(
    result_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    app.reorder_pinned_results(result_ids)
        .map_err(|error| error.to_error_string())
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
            .map_err(|error| error.to_error_string())?
    };

    match execution.effect {
        CommandEffect::QuitApp => app_handle.exit(0),
        CommandEffect::OpenLogsDirectory => {
            let log_dir = log_directory(&app_handle)?;
            app_handle.opener().open_path(
                log_dir.to_string_lossy().as_ref(),
                None::<&str>,
            ).map_err(|e| e.to_string())?;
        }
        CommandEffect::OpenDataDirectory => {
            let data_dir = state.data_dir().to_path_buf();
            app_handle.opener().open_path(
                data_dir.to_string_lossy().as_ref(),
                None::<&str>,
            ).map_err(|e| e.to_string())?;
        }
        // 前端已改由 openPluginView 驱动——先调 IPC 获取 placement，
        // docked 才 navigate(route)，后端不再在这里操作窗口。
        CommandEffect::OpenPluginView { .. } => {}
        _ => {}
    }

    Ok(execution)
}

fn log_directory(app_handle: &AppHandle) -> Result<std::path::PathBuf, String> {
    app_handle
        .path()
        .app_log_dir()
        .map_err(|error| error.to_string())
}
