use std::{path::Path, process::Command};

use litools_core::{CommandEffect, CommandExecution, LauncherPanelResponse};
use litools_search::SearchResult;
use tauri::{AppHandle, Manager, State};

use crate::{state::AppState, surface::service};

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
        CommandEffect::QuitApp => app_handle.exit(0),
        CommandEffect::OpenSettings => {
            service::open_view_route(&app_handle, &state, "/settings", state.center_on_show())?;
        }
        CommandEffect::OpenDiagnostics => {
            service::open_view_route(&app_handle, &state, "/diagnostics", state.center_on_show())?;
        }
        CommandEffect::OpenPlugins => {
            service::open_view_route(&app_handle, &state, "/plugins", state.center_on_show())?;
        }
        CommandEffect::OpenLogsDirectory => {
            open_directory(&log_directory(&app_handle)?)?;
        }
        CommandEffect::OpenDataDirectory => {
            open_directory(state.data_dir())?;
        }
        CommandEffect::OpenPluginView { ref route, .. } => {
            service::open_view_route(&app_handle, &state, route, state.center_on_show())?;
        }
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

fn open_directory(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|error| error.to_string())?;

    let status = open_directory_command(path)
        .status()
        .map_err(|error| error.to_string())?;

    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("打开目录失败：{}", path.display()))
}

#[cfg(target_os = "macos")]
fn open_directory_command(path: &Path) -> Command {
    let mut command = Command::new("open");
    command.arg(path);
    command
}

#[cfg(target_os = "windows")]
fn open_directory_command(path: &Path) -> Command {
    let mut command = Command::new("explorer");
    command.arg(path);
    command
}

#[cfg(target_os = "linux")]
fn open_directory_command(path: &Path) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(path);
    command
}
