mod app_watcher;
mod index_refresh;
mod ipc;
#[cfg(target_os = "macos")]
mod macos_icon;
mod plugin_runtime;
mod protocol;
mod shortcut;
mod state;
mod surface;
mod tray;
mod view;
mod windowing;

use litools_core::AppBootstrapPaths;
use state::AppState;
use tauri::Manager;
use tauri_plugin_global_shortcut::ShortcutState;

fn bundled_plugins_dir(app: &tauri::App) -> Option<std::path::PathBuf> {
    // In dev mode, prefer the source tree path so that newly created plugins
    // are picked up without a full Tauri resource rebuild. The resolved
    // resource path (target/debug/plugins/bundled) is a build-time snapshot.
    let dev_path = dev_bundled_plugins_dir();
    if dev_path.is_some() {
        return dev_path;
    }
    app.path()
        .resolve("plugins/bundled", tauri::path::BaseDirectory::Resource)
        .ok()
        .filter(|path| path.exists())
}

#[cfg(debug_assertions)]
fn dev_bundled_plugins_dir() -> Option<std::path::PathBuf> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("plugins/bundled");
    path.exists().then_some(path)
}

#[cfg(not(debug_assertions))]
fn dev_bundled_plugins_dir() -> Option<std::path::PathBuf> {
    None
}

fn main() {
    let icon_protocol = protocol::IconProtocol::default();
    let plugin_protocol = protocol::PluginProtocol;

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
        .register_uri_scheme_protocol(
            protocol::plugin::PLUGIN_PROTOCOL_SCHEME,
            move |context, request| {
                let Some(state) = context.app_handle().try_state::<AppState>() else {
                    return tauri::http::Response::builder()
                        .status(tauri::http::StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Vec::new())
                        .expect("valid plugin protocol error response");
                };

                plugin_protocol.handle(&state, request.uri())
            },
        )
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
            let bundled_plugins_dir = bundled_plugins_dir(app);
            app.manage(AppState::bootstrap(AppBootstrapPaths {
                data_dir,
                bundled_plugins_dir,
            })?);
            surface::service::bootstrap_main_surface(app.handle(), &app.state::<AppState>())?;
            plugin_runtime::service::warm_titlebar_pool(
                app.handle(),
                &app.state::<AppState>(),
            );
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
            ipc::surface::focus_main_window,
            ipc::surface::resize_main_window_height,
            ipc::diagnostics::reload_index,
            ipc::settings::get_settings,
            ipc::settings::update_settings,
            ipc::plugins::list_plugins,
            ipc::plugins::get_plugin_view_descriptor,
            plugin_runtime::ipc::open_plugin_view,
            plugin_runtime::ipc::hide_plugin_view,
            plugin_runtime::ipc::detach_plugin_view,
            plugin_runtime::ipc::close_plugin_view,
            plugin_runtime::ipc::close_plugin_view_by_id,
            plugin_runtime::ipc::get_plugin_view_info,
            plugin_runtime::ipc::plugin_view_call,
            ipc::diagnostics::get_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("failed to run litools");
}
