// litools-sdk Tauri 插件：所有插件（含第三方）都能调用的能力

use serde_json::{Value, json};
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{AppHandle, State, Webview, ipc::InvokeError};

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
            sdk_commands_add,
            sdk_commands_remove,
            sdk_commands_replace,
            sdk_commands_update,
            sdk_search_register_provider,
            sdk_search_unregister_provider,
            sdk_search_submit,
            sdk_input_register_detector,
            sdk_input_unregister_detector,
            sdk_detection_submit,
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
sdk_cmd!(sdk_commands_add, "commands.add", commands: Value);
sdk_cmd!(sdk_commands_remove, "commands.remove", ids: Value);
sdk_cmd!(sdk_commands_replace, "commands.replace", commands: Value);
sdk_cmd!(sdk_commands_update, "commands.update", id: String, cmd: Value);
sdk_cmd!(sdk_search_register_provider, "search.registerProvider", id: String, timeout: Option<u64>);
sdk_cmd!(sdk_search_unregister_provider, "search.unregisterProvider", id: String);
sdk_cmd!(sdk_input_register_detector, "input.registerDetector", id: String, feature_kind: Option<String>, timeout: Option<u64>);
sdk_cmd!(sdk_input_unregister_detector, "input.unregisterDetector", id: String);

/// Internal Protocol: 插件 WebView 回传搜索结果
#[tauri::command]
fn sdk_search_submit(
    request_id: String,
    results: Vec<litools_search::SearchResult>,
    webview: Webview,
    state: State<'_, AppState>,
) -> Result<Value, InvokeError> {
    let Some(runtime_id) = runtime_id_for_webview(&webview, &state) else {
        return Ok(Value::Null);
    };

    // 解析 SearchRequestId: "provider_id.nonce"
    let parts: Vec<&str> = request_id.rsplitn(2, '.').collect();
    if parts.len() != 2 {
        return Ok(Value::Null);
    }
    let (nonce_str, provider_id) = (parts[0], parts[1]);
    let Ok(nonce) = uuid::Uuid::parse_str(nonce_str) else {
        return Ok(Value::Null);
    };

    let sid =
        crate::core::plugins::runtime::search_bridge::SearchRequestId::new(provider_id, nonce);

    state.search_bridge.complete(&sid, &runtime_id, results);
    Ok(Value::Null)
}

/// Internal Protocol: 插件 WebView 回传检测结果（Phase 4D）
#[tauri::command]
fn sdk_detection_submit(
    request_id: String,
    detection: Option<litools_search::Detection>,
    webview: Webview,
    state: State<'_, AppState>,
) -> Result<Value, InvokeError> {
    let Some(runtime_id) = runtime_id_for_webview(&webview, &state) else {
        return Ok(Value::Null);
    };

    // 解析 DetectionRequestId: "detector_id.nonce"
    let parts: Vec<&str> = request_id.rsplitn(2, '.').collect();
    if parts.len() != 2 {
        return Ok(Value::Null);
    }
    let (nonce_str, detector_id) = (parts[0], parts[1]);
    let Ok(nonce) = uuid::Uuid::parse_str(nonce_str) else {
        return Ok(Value::Null);
    };

    let did = crate::core::plugins::runtime::detection_bridge::DetectionRequestId {
        detector_id: detector_id.to_string(),
        nonce,
    };

    state
        .detection_bridge
        .complete(&did, &runtime_id, detection);
    Ok(Value::Null)
}

fn runtime_id_for_webview(webview: &Webview, state: &AppState) -> Option<String> {
    state
        .plugin_runtimes
        .lock()
        .unwrap()
        .runtime_for_surface_id(webview.label())
        .map(|context| context.id)
}
