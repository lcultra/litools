// litools-sdk Tauri 插件：所有插件（含第三方）都能调用的能力

use serde_json::{Value, json};
use tauri::{AppHandle, State, Webview, ipc::InvokeError};
use tauri::plugin::{Builder, TauriPlugin};

use crate::core::plugins::runtime::commands::route_plugin_view_call_inner;
use crate::state::AppState;

pub const SDK_PLUGIN: &str = "litools-sdk";

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new(SDK_PLUGIN)
        .invoke_handler(tauri::generate_handler![
            sdk_runtime_ready,
            sdk_runtime_get_info,
            sdk_permissions_query,
            sdk_ui_close,
            sdk_ui_set_title,
            sdk_ui_toast,
            sdk_storage_get,
            sdk_storage_set,
            sdk_storage_remove,
            sdk_storage_clear,
            sdk_settings_get,
            sdk_settings_update,
            sdk_diagnostics_get,
            sdk_plugins_list,
        ])
        .build()
}

macro_rules! sdk_cmd {
    ($name:ident, $method:literal) => {
        #[tauri::command]
        fn $name(
            app_handle: AppHandle,
            state: State<'_, AppState>,
            webview: Webview,
        ) -> Result<Value, InvokeError> {
            route_plugin_view_call_inner($method, json!({}), &webview, &state, &app_handle)
                .map_err(InvokeError::from)
        }
    };
    ($name:ident, $method:literal, $($pname:ident: $ptype:ty),+) => {
        #[tauri::command]
        fn $name(
            $($pname: $ptype,)*
            app_handle: AppHandle,
            state: State<'_, AppState>,
            webview: Webview,
        ) -> Result<Value, InvokeError> {
            let mut p = json!({});
            $(p[stringify!($pname)] = json!($pname);)*
            route_plugin_view_call_inner($method, p, &webview, &state, &app_handle)
                .map_err(InvokeError::from)
        }
    };
}

sdk_cmd!(sdk_runtime_ready, "runtime.ready");
sdk_cmd!(sdk_runtime_get_info, "runtime.getInfo");
sdk_cmd!(sdk_permissions_query, "permissions.query", permission: String);
sdk_cmd!(sdk_ui_close, "ui.close");
sdk_cmd!(sdk_ui_set_title, "ui.setTitle", title: String);
sdk_cmd!(sdk_ui_toast, "ui.toast", message: String, options: Option<Value>);
sdk_cmd!(sdk_storage_get, "storage.get", key: String);
sdk_cmd!(sdk_storage_set, "storage.set", key: String, value: Value);
sdk_cmd!(sdk_storage_remove, "storage.remove", key: String);
sdk_cmd!(sdk_storage_clear, "storage.clear");
sdk_cmd!(sdk_settings_get, "settings.get");
sdk_cmd!(sdk_settings_update, "settings.update", settings: Value);
sdk_cmd!(sdk_diagnostics_get, "diagnostics.get");
sdk_cmd!(sdk_plugins_list, "plugins.list");
