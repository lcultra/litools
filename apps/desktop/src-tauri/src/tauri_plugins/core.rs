use tauri::plugin::{Builder, TauriPlugin};

use super::constants::CORE_PLUGIN;

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new(CORE_PLUGIN)
        .invoke_handler(tauri::generate_handler![
            crate::ipc::launcher::search,
            crate::ipc::launcher::launcher_panel,
            crate::ipc::launcher::pin_result,
            crate::ipc::launcher::unpin_result,
            crate::ipc::launcher::reorder_pinned_results,
            crate::ipc::launcher::execute_result,
            crate::ipc::surface::detach_route,
            crate::ipc::surface::update_surface_route,
            crate::ipc::surface::list_windows,
            crate::ipc::surface::get_current_window_metadata,
            crate::ipc::surface::hide_window,
            crate::ipc::surface::focus_window,
            crate::ipc::surface::destroy_window,
            crate::ipc::surface::start_window_dragging,
            crate::ipc::surface::hide_main_window,
            crate::ipc::surface::show_main_window,
            crate::ipc::surface::focus_main_window,
            crate::ipc::surface::resize_main_window_height,
            crate::ipc::surface::reveal_in_file_manager,
            crate::ipc::diagnostics::reload_index,
            crate::ipc::diagnostics::get_diagnostics,
            crate::ipc::settings::get_settings,
            crate::ipc::settings::update_settings,
            crate::ipc::plugins::list_plugins,
            crate::ipc::plugins::get_plugin_view_descriptor,
            crate::plugin_runtime::ipc::open_plugin_view,
            crate::plugin_runtime::ipc::hide_plugin_view,
            crate::plugin_runtime::ipc::detach_plugin_view,
            crate::plugin_runtime::ipc::close_plugin_view,
            crate::plugin_runtime::ipc::close_plugin_view_by_id,
            crate::plugin_runtime::ipc::get_plugin_view_info,
            crate::plugin_runtime::ipc::open_plugin_devtools,
        ])
        .build()
}
