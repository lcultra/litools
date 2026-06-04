mod commands;
mod state;

use state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(AppState::bootstrap(data_dir)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::execute_result,
            commands::get_settings,
            commands::list_plugins,
            commands::get_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("failed to run litools");
}
