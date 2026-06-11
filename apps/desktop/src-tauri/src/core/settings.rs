use litools_settings::AppSettings;
use tauri::{AppHandle, State};

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
    Ok(updated_settings)
}
