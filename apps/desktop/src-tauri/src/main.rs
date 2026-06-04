mod commands;
mod state;

use state::AppState;

fn main() {
    let app_state = AppState::bootstrap().expect("failed to bootstrap litools");

    tauri::Builder::default()
        .manage(app_state)
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
