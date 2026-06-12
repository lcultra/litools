use litools_settings::AppSettings;
use tauri::{AppHandle, Manager, State};

use crate::{shortcut, state::AppState};

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
            .map_err(|error| error.to_error_string())?
    };

    shortcut::register_global_shortcut(&app_handle, &updated_settings.palette.global_hotkey);
    apply_theme_to_all_windows(&app_handle, &updated_settings.theme);
    Ok(updated_settings)
}

fn to_tauri_theme(theme: &str) -> Option<tauri::Theme> {
    match theme {
        "dark" => Some(tauri::Theme::Dark),
        "light" => Some(tauri::Theme::Light),
        _ => None,
    }
}

pub fn apply_theme_to_all_windows(app_handle: &AppHandle, theme: &str) {
    let tauri_theme = to_tauri_theme(theme);
    let windows = app_handle.windows();
    log::info!("windows 列表 ({} 个): {:?}", windows.len(), windows.keys().collect::<Vec<_>>());
    for (label, window) in &windows {
        log::info!("  set_theme({:?}) on label={}", tauri_theme, label);
        let _ = window.set_theme(tauri_theme);
    }
    log::info!("主题已应用到所有窗口: {}", theme);
}
