mod app_watcher;
mod core;
mod index_refresh;
#[cfg(target_os = "macos")]
mod macos_icon;
mod sdk;
mod protocol;
mod shortcut;
mod state;
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
                        let _ = core::surface::service::toggle_main_launcher(app, &state);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_upload::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_authenticator::init())
        .plugin(tauri_plugin_geolocation::init())
        .plugin(tauri_plugin_haptics::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .max_file_size(2_000_000)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
                .build(),
        )
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(core::init())
        .plugin(sdk::init())
        .setup(move |app| {
            let data_dir = app.path().app_data_dir()?;
            let bundled_plugins_dir = bundled_plugins_dir(app);
            app.manage(AppState::bootstrap(AppBootstrapPaths {
                data_dir,
                bundled_plugins_dir,
            })?);
            core::surface::service::bootstrap_main_surface(app.handle(), &app.state::<AppState>())?;
            core::plugins::runtime::service::warm_detached_pool(app.handle(), &app.state::<AppState>());
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
            log::info!("应用已就绪");
            Ok(())
        })
        .on_window_event(windowing::lifecycle::handle_window_event)
        .run(tauri::generate_context!())
        .expect("failed to run litools");
}
