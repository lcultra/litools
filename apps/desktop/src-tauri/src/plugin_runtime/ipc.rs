use chrono::Utc;
use litools_index::repository::PluginStorageRepository;
use litools_settings::AppSettings;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Webview, ipc::InvokeError};

use crate::{
    plugin_runtime::{
        model::{PermissionQueryResult, PluginRuntimeError, PluginRuntimeInfo},
        permissions, service,
    },
    shortcut,
    state::AppState,
};
pub use litools_config::plugin::{MAX_STORAGE_KEY_LEN, MAX_STORAGE_VALUE_LEN};

#[tauri::command]
pub fn open_plugin_view(
    plugin_id: String,
    command_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<PluginRuntimeInfo, String> {
    service::dock_plugin_runtime(&app_handle, &state, &plugin_id, &command_id)
        .map(|context| PluginRuntimeInfo::from(&context))
}

#[tauri::command]
pub fn hide_plugin_view(
    plugin_id: String,
    command_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<PluginRuntimeInfo, String> {
    service::hide_docked_plugin_runtime(&app_handle, &state, &plugin_id, &command_id)
        .map(|context| PluginRuntimeInfo::from(&context))
}

#[tauri::command]
pub fn detach_plugin_view(
    plugin_id: String,
    command_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<PluginRuntimeInfo, String> {
    service::detach_plugin_runtime(&app_handle, &state, &plugin_id, &command_id)
        .map(|context| PluginRuntimeInfo::from(&context))
}

#[tauri::command]
pub fn close_plugin_view(
    plugin_id: String,
    command_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    service::close_plugin_runtime_by_plugin_command(&app_handle, &state, &plugin_id, &command_id)
}

#[tauri::command]
pub fn close_plugin_view_by_id(
    runtime_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    service::close_runtime(&app_handle, &state, &runtime_id)
}

#[tauri::command]
pub fn get_plugin_view_info(
    runtime_id: String,
    state: State<'_, AppState>,
) -> Result<PluginRuntimeInfo, String> {
    service::runtime_info(&state, &runtime_id)
}

/// 打开插件 webview 的开发者工具（仅开发模式）
#[tauri::command]
pub fn open_plugin_devtools(
    runtime_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let context = state
        .plugin_runtime(&runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    let webview = app_handle
        .get_webview(&context.webview_label)
        .ok_or_else(|| format!("webview not found: {}", context.webview_label))?;
    webview.open_devtools();
    Ok(())
}

#[tauri::command]
pub fn plugin_view_call(
    method: String,
    params: Value,
    webview: Webview,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<Value, InvokeError> {
    route_plugin_view_call(&method, params, &webview, &state, &app_handle)
        .map_err(InvokeError::from)
}

fn route_plugin_view_call(
    method: &str,
    params: Value,
    webview: &Webview,
    state: &AppState,
    app_handle: &AppHandle,
) -> Result<Value, PluginRuntimeError> {
    if !crate::windowing::labels::is_plugin_webview_label(webview.label()) {
        return Err(PluginRuntimeError::permission_denied(format!(
            "not a plugin runtime webview: {}",
            webview.label()
        )));
    }

    let context = state
        .plugin_runtime_for_webview_label(webview.label())
        .ok_or_else(|| {
            PluginRuntimeError::permission_denied(format!(
                "not a registered plugin runtime webview: {}",
                webview.label()
            ))
        })?;

    if !permissions::can_call_method(&context, method) {
        return Err(PluginRuntimeError::permission_denied(format!(
            "plugin {} is not allowed to call {method}",
            context.plugin_id
        )));
    }

    match method {
        "runtime.ready" => {
            let context = service::mark_runtime_ready(app_handle, state, &context.id)
                .map_err(PluginRuntimeError::internal)?;
            Ok(json!(PluginRuntimeInfo::from(&context)))
        }
        "runtime.getInfo" => Ok(json!(PluginRuntimeInfo::from(&context))),
        "permissions.query" => {
            let permission = required_string_param(&params, "permission")?;
            Ok(json!(PermissionQueryResult {
                permission: permission.clone(),
                state: permissions::query_permission(&context, &permission),
            }))
        }
        "permissions.check" => {
            let command = params
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let allowed = permissions::check_toplevel_invoke(&context, command);
            Ok(json!({ "allowed": allowed }))
        }
        "ui.close" => {
            service::close_runtime(app_handle, state, &context.id)
                .map_err(PluginRuntimeError::internal)?;
            Ok(Value::Null)
        }
        "ui.setTitle" => {
            let title = required_string_param(&params, "title")?;
            validate_title(&title)?;
            service::mark_runtime_title(app_handle, state, &context.id, title)
                .map_err(PluginRuntimeError::internal)?;
            Ok(Value::Null)
        }
        "ui.toast" => Err(PluginRuntimeError::unsupported(
            "ui.toast is not connected to a host toast presenter yet",
        )),
        "storage.get" => {
            let key = storage_key_param(&params)?;
            let value_json = with_storage(state, |repository| {
                repository.get_json(&context.plugin_id, &key)
            })?;
            match value_json {
                Some(value_json) => serde_json::from_str(&value_json)
                    .map_err(|error| PluginRuntimeError::internal(error.to_string())),
                None => Ok(Value::Null),
            }
        }
        "storage.set" => {
            let key = storage_key_param(&params)?;
            let value = params
                .get("value")
                .ok_or_else(|| PluginRuntimeError::invalid_params("storage.set requires value"))?;
            let value_json = serde_json::to_string(value)
                .map_err(|error| PluginRuntimeError::invalid_params(error.to_string()))?;
            if value_json.len() > MAX_STORAGE_VALUE_LEN {
                return Err(PluginRuntimeError::invalid_params(format!(
                    "storage value exceeds {MAX_STORAGE_VALUE_LEN} bytes"
                )));
            }
            with_storage(state, |repository| {
                repository.set_json(
                    &context.plugin_id,
                    &key,
                    &value_json,
                    &Utc::now().to_rfc3339(),
                )
            })?;
            Ok(Value::Null)
        }
        "storage.remove" => {
            let key = storage_key_param(&params)?;
            with_storage(state, |repository| {
                repository.remove(&context.plugin_id, &key)
            })?;
            Ok(Value::Null)
        }
        "storage.clear" => {
            with_storage(state, |repository| repository.clear(&context.plugin_id))?;
            Ok(Value::Null)
        }
        "settings.get" => {
            let app = state
                .app()
                .lock()
                .map_err(|error| PluginRuntimeError::internal(error.to_string()))?;
            let settings = app.settings().clone();
            Ok(json!(settings))
        }
        "settings.update" => {
            let new_settings: AppSettings =
                serde_json::from_value(params.get("settings").cloned().unwrap_or(Value::Null))
                    .map_err(|error| PluginRuntimeError::invalid_params(error.to_string()))?;
            let updated_settings = {
                let mut app = state
                    .app()
                    .lock()
                    .map_err(|error| PluginRuntimeError::internal(error.to_string()))?;
                app.update_settings(new_settings)
                    .map_err(|error| PluginRuntimeError::internal(error.to_string()))?
            };
            shortcut::register_global_shortcut(app_handle, &updated_settings.palette.global_hotkey);
            Ok(json!(updated_settings))
        }
        "diagnostics.get" => Ok(json!(
            crate::ipc::diagnostics::get_diagnostics_inner(state,)
                .map_err(|error| PluginRuntimeError::internal(error))?
        )),
        "plugins.list" => {
            let app = state
                .app()
                .lock()
                .map_err(|error| PluginRuntimeError::internal(error.to_string()))?;
            let plugins: Vec<litools_plugin::manager::InstalledPlugin> = app
                .context()
                .plugins
                .installed_plugins()
                .into_iter()
                .cloned()
                .collect();
            Ok(json!(plugins
                .iter()
                .map(|plugin| serde_json::json!({
                    "id": plugin.manifest.id,
                    "name": plugin.manifest.name,
                    "version": plugin.manifest.version,
                    "description": plugin.manifest.description,
                    "author": plugin.manifest.author,
                    "icon": plugin.manifest.icon,
                    "enabled": plugin.enabled,
                    "trusted": plugin.trusted,
                    "source": plugin.source.as_str(),
                    "path": plugin.path.to_string_lossy(),
                    "permissions": plugin.manifest.permissions,
                    "commands": plugin.manifest.commands.iter().map(|command| serde_json::json!({
                        "id": command.id,
                        "title": command.title,
                        "subtitle": command.subtitle,
                        "keywords": command.keywords,
                        "mode": command.mode.as_str(),
                    })).collect::<Vec<_>>(),
                }))
                .collect::<Vec<_>>()))
        }
        _ => Err(PluginRuntimeError::permission_denied(format!(
            "unknown plugin runtime method: {method}"
        ))),
    }
}

fn with_storage<T>(
    state: &AppState,
    operation: impl FnOnce(&PluginStorageRepository<'_>) -> rusqlite::Result<T>,
) -> Result<T, PluginRuntimeError> {
    let app = state
        .app()
        .lock()
        .map_err(|error| PluginRuntimeError::internal(error.to_string()))?;
    let connection = app.context().database.connection();
    operation(&PluginStorageRepository::new(&connection))
        .map_err(|error| PluginRuntimeError::internal(error.to_string()))
}

fn required_string_param(params: &Value, key: &str) -> Result<String, PluginRuntimeError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| PluginRuntimeError::invalid_params(format!("{key} must be a string")))
}

fn storage_key_param(params: &Value) -> Result<String, PluginRuntimeError> {
    let key = required_string_param(params, "key")?;
    if key.is_empty() {
        return Err(PluginRuntimeError::invalid_params(
            "storage key must not be empty",
        ));
    }
    if key.len() > MAX_STORAGE_KEY_LEN {
        return Err(PluginRuntimeError::invalid_params(format!(
            "storage key exceeds {MAX_STORAGE_KEY_LEN} bytes"
        )));
    }
    if key.chars().any(char::is_control) {
        return Err(PluginRuntimeError::invalid_params(
            "storage key must not contain control characters",
        ));
    }
    Ok(key)
}

fn validate_title(title: &str) -> Result<(), PluginRuntimeError> {
    if title.trim().is_empty() {
        return Err(PluginRuntimeError::invalid_params(
            "title must not be empty",
        ));
    }
    if title.len() > 128 {
        return Err(PluginRuntimeError::invalid_params(
            "title must not exceed 128 bytes",
        ));
    }
    Ok(())
}
