mod app_watcher;
mod commands;
mod icon_cache;
mod icon_protocol;
mod index_refresh;
#[cfg(target_os = "macos")]
mod macos_icon;
mod shortcut;
mod state;
mod tray;
mod window;

use state::AppState;
use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;

fn main() {
    let icon_protocol = icon_protocol::IconProtocol::default();

    tauri::Builder::default()
        .register_uri_scheme_protocol("litools-icon", move |context, request| {
            let Some(state) = context.app_handle().try_state::<AppState>() else {
                return tauri::http::Response::builder()
                    .status(tauri::http::StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Vec::new())
                    .expect("valid icon protocol error response");
            };

            icon_protocol.handle(&state, request.uri())
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    let Some(state) = app.try_state::<AppState>() else {
                        return;
                    };

                    if event.state() == ShortcutState::Pressed
                        && shortcut::matches_palette_shortcut(shortcut, &state)
                        && let Some(window) = window::main_window(app)
                    {
                        window::toggle_main_window(&window, state.center_on_show());
                    }
                })
                .build(),
        )
        .setup(move |app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(AppState::bootstrap(data_dir)?);
            tray::setup_tray(app)?;
            shortcut::register_global_shortcut(
                app.handle(),
                &app.state::<AppState>().global_hotkey(),
            );
            let app_watcher = app_watcher::start_app_watcher(app.handle().clone());
            app.state::<AppState>().set_app_watcher(app_watcher);
            index_refresh::request_index_refresh(
                app.handle(),
                index_refresh::IndexRefreshTrigger::Startup,
            );
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != window::MAIN_WINDOW_LABEL {
                return;
            }

            let Some(state) = window.try_state::<AppState>() else {
                return;
            };

            match event {
                WindowEvent::CloseRequested { api, .. } if !state.is_quitting() => {
                    if state.close_to_tray() {
                        api.prevent_close();
                        let _ = window.hide();
                    } else {
                        state.request_quit();
                    }
                }
                WindowEvent::Focused(false) if !state.is_quitting() && state.hide_on_blur() => {
                    let _ = window.hide();
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::launcher_panel,
            commands::pin_result,
            commands::unpin_result,
            commands::reorder_pinned_results,
            commands::execute_result,
            commands::hide_main_window,
            commands::show_main_window,
            commands::focus_main_window,
            commands::start_dragging,
            commands::resize_main_window_height,
            commands::reload_index,
            commands::get_settings,
            commands::update_settings,
            commands::list_plugins,
            commands::get_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("failed to run litools");
}
