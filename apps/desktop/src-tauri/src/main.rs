mod commands;
mod state;
mod window;

use state::AppState;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

fn main() {
    let palette_shortcut = Shortcut::new(Some(Modifiers::META), Code::Space);

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if shortcut == &palette_shortcut
                        && event.state() == ShortcutState::Pressed
                        && let Some(window) = window::main_window(app)
                    {
                        window::toggle_main_window(&window);
                    }
                })
                .build(),
        )
        .setup(move |app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(AppState::bootstrap(data_dir)?);
            app.global_shortcut().register(palette_shortcut)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::execute_result,
            commands::hide_main_window,
            commands::show_main_window,
            commands::get_settings,
            commands::list_plugins,
            commands::get_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("failed to run litools");
}
