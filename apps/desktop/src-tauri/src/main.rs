mod app_watcher;
mod icon_cache;
mod icon_protocol;
mod index_refresh;
mod ipc;
#[cfg(target_os = "macos")]
mod macos_icon;
mod shortcut;
mod state;
mod surface;
mod tray;
mod view;
mod windowing;

use state::AppState;
use tauri::Manager;
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
                    {
                        let _ = surface::service::toggle_main_launcher(app, &state);
                    }
                })
                .build(),
        )
        .setup(move |app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(AppState::bootstrap(data_dir)?);
            surface::service::bootstrap_main_surface(app.handle(), &app.state::<AppState>())?;
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
        .on_window_event(windowing::lifecycle::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            ipc::launcher::search,
            ipc::launcher::launcher_panel,
            ipc::launcher::pin_result,
            ipc::launcher::unpin_result,
            ipc::launcher::reorder_pinned_results,
            ipc::launcher::execute_result,
            ipc::surface::detach_route,
            ipc::surface::update_surface_route,
            ipc::surface::open_route,
            ipc::surface::list_windows,
            ipc::surface::get_current_window_metadata,
            ipc::surface::hide_window,
            ipc::surface::focus_window,
            ipc::surface::destroy_window,
            ipc::surface::start_window_dragging,
            ipc::surface::hide_main_window,
            ipc::surface::show_main_window,
            ipc::surface::open_settings,
            ipc::surface::focus_main_window,
            ipc::surface::start_dragging,
            ipc::surface::resize_main_window_height,
            ipc::diagnostics::reload_index,
            ipc::settings::get_settings,
            ipc::settings::update_settings,
            ipc::diagnostics::list_plugins,
            ipc::diagnostics::get_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("failed to run litools");
}
