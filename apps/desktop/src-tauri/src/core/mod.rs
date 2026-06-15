// litools-core Tauri 插件：主应用 + 内置(trusted)插件的能力

pub mod diagnostics;
pub mod events;
pub mod executor;
pub mod launcher;
pub mod plugins;
pub mod settings;
pub mod surface;

use tauri::plugin::{Builder, TauriPlugin};

/// litools 内置插件命名常量
pub const CORE_PLUGIN: &str = "litools-core";

/// ACL permission 前缀
pub const CORE_PREFIX: &str = "litools-core:";
pub const SDK_PREFIX: &str = "litools-sdk:";

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new(CORE_PLUGIN)
        .invoke_handler(tauri::generate_handler![
            crate::core::launcher::search,
            crate::core::launcher::launcher_panel,
            crate::core::launcher::pin_result,
            crate::core::launcher::unpin_result,
            crate::core::launcher::reorder_pinned_results,
            crate::core::launcher::execute_result,
            crate::core::surface::commands::get_base_info,
            crate::core::surface::commands::detach_route,
            crate::core::surface::commands::update_surface_route,
            crate::core::surface::commands::list_windows,
            crate::core::surface::commands::get_current_window_metadata,
            crate::core::surface::commands::hide_window,
            crate::core::surface::commands::focus_window,
            crate::core::surface::commands::destroy_window,
            crate::core::surface::commands::start_window_dragging,
            crate::core::surface::commands::hide_main_window,
            crate::core::surface::commands::show_main_window,
            crate::core::surface::commands::focus_main_window,
            crate::core::surface::commands::resize_main_window_height,
            crate::core::surface::commands::reveal_in_file_manager,
            crate::core::diagnostics::reload_index,
            crate::core::diagnostics::get_diagnostics,
            crate::core::settings::get_settings,
            crate::core::settings::update_settings,
            crate::core::plugins::commands::list_plugins,
            crate::core::plugins::commands::get_plugin_view_descriptor,
            crate::core::plugins::commands::install_plugin,
            crate::core::plugins::commands::uninstall_plugin,
            crate::core::plugins::commands::toggle_plugin,
            crate::core::plugins::commands::add_commands,
            crate::core::plugins::commands::remove_commands,
            crate::core::plugins::commands::replace_commands,
            crate::core::plugins::runtime::commands::open_plugin_view,
            crate::core::plugins::runtime::commands::hide_plugin_view,
            crate::core::plugins::runtime::commands::detach_plugin_view,
            crate::core::plugins::runtime::commands::close_plugin_view,
            crate::core::plugins::runtime::commands::close_plugin_view_by_id,
            crate::core::plugins::runtime::commands::get_plugin_view_info,
            crate::core::plugins::runtime::commands::open_plugin_devtools,
        ])
        .build()
}
