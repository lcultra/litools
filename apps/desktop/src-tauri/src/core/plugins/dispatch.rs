use chrono::Utc;
use litools_index::repository::PluginStorageRepository;
use litools_settings::AppSettings;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Webview};

use crate::{
    core::plugins::runtime::{
        model::{PermissionQueryResult, PluginRuntimeError},
        permissions, service,
    },
    shortcut,
    state::AppState,
};
pub use litools_config::plugin::{MAX_STORAGE_KEY_LEN, MAX_STORAGE_VALUE_LEN};

/// SDK 方法路由：根据 method 字符串分发到实际的业务逻辑。
pub fn route_plugin_view_call(
    method: &str,
    params: Value,
    webview: &Webview,
    state: &AppState,
    app_handle: &AppHandle,
) -> Result<Value, PluginRuntimeError> {
    log::debug!("[dispatch] {method} from webview={}", webview.label());
    // 通过 webview_label（即 surface_id）直接查找 runtime context
    let context = state
        .plugin_runtimes
        .lock().unwrap()
        .runtime_for_surface_id(webview.label())
        .ok_or_else(|| {
            PluginRuntimeError::permission_denied(format!(
                "not a registered plugin runtime: {}",
                webview.label()
            ))
        })?;

    if !permissions::can_call_method(&context, method) {
        return Err(PluginRuntimeError::permission_denied(format!(
            "plugin {} is not allowed to call {method}",
            context.plugin_id
        )));
    }

    use crate::core::plugins::runtime::method_registry::{MethodDescriptor, MethodId};

    let Some(desc) = MethodDescriptor::find_by_name(method) else {
        return Err(PluginRuntimeError::permission_denied(format!(
            "unknown plugin runtime method: {method}"
        )));
    };

    match desc.id {
        MethodId::RuntimeReady => {
            let context = service::mark_runtime_ready(app_handle, state, &context.id)
                .map_err(PluginRuntimeError::internal)?;
            Ok(json!(service::build_runtime_info(state, &context)))
        }
        MethodId::RuntimeGetInfo => Ok(json!(service::build_runtime_info(state, &context))),
        MethodId::RuntimeQueryPermission => {
            let permission = required_string_param(&params, "permission")?;
            Ok(json!(PermissionQueryResult {
                permission: permission.clone(),
                state: permissions::query_permission(&context, &permission),
            }))
        }
        MethodId::UIClose => {
            service::close_runtime(app_handle, state, &context.id)
                .map_err(PluginRuntimeError::internal)?;
            Ok(Value::Null)
        }
        MethodId::UISetTitle => {
            let title = required_string_param(&params, "title")?;
            validate_title(&title)?;
            service::mark_runtime_title(app_handle, state, &context.id, title)
                .map_err(PluginRuntimeError::internal)?;
            Ok(Value::Null)
        }
        MethodId::UIToast => {
            let message = required_string_param(&params, "message")?;
            let payload = json!({
                "pluginId": context.plugin_id,
                "message": message,
                "options": params.get("options").cloned().unwrap_or(Value::Null),
            });
            let _ = app_handle.emit("litools:toast", payload);
            Ok(Value::Null)
        }
        MethodId::StorageGet => {
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
        MethodId::StorageSet => {
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
        MethodId::StorageRemove => {
            let key = storage_key_param(&params)?;
            with_storage(state, |repository| {
                repository.remove(&context.plugin_id, &key)
            })?;
            Ok(Value::Null)
        }
        MethodId::StorageClear => {
            with_storage(state, |repository| repository.clear(&context.plugin_id))?;
            Ok(Value::Null)
        }
        MethodId::SettingsGet => {
            let app = state
                .app()
                .read().unwrap();
            let settings = app.settings().clone();
            Ok(json!(settings))
        }
        MethodId::SettingsUpdate => {
            let new_settings: AppSettings =
                serde_json::from_value(params.get("settings").cloned().unwrap_or(Value::Null))
                    .map_err(|error| PluginRuntimeError::invalid_params(error.to_string()))?;
            let updated_settings = {
                let mut app = state
                    .app()
                    .write().unwrap();
                app.update_settings(new_settings)
                    .map_err(|error| PluginRuntimeError::internal(error.to_string()))?
            };
            shortcut::register_global_shortcut(app_handle, &updated_settings.palette.global_hotkey);
            crate::core::settings::apply_theme_to_all_windows(app_handle, &updated_settings.theme);
            Ok(json!(updated_settings))
        }
        MethodId::DiagnosticsGet => Ok(json!(
            crate::core::diagnostics::get_diagnostics_inner(state,)
                .map_err(|error| PluginRuntimeError::internal(error))?
        )),
        MethodId::HostPluginsList => {
            let app = state
                .app()
                .read().unwrap();
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
        MethodId::CommandsAdd => {
            let commands: Vec<Value> = params
                .get("commands")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            crate::core::plugins::commands::add_commands_inner(state, webview, commands)
                .map_err(|e| PluginRuntimeError::internal(e))?;
            Ok(Value::Null)
        }
        MethodId::CommandsRemove => {
            let ids: Vec<String> = params
                .get("ids")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            crate::core::plugins::commands::remove_commands_inner(state, webview, ids)
                .map_err(|e| PluginRuntimeError::internal(e))?;
            Ok(Value::Null)
        }
        MethodId::CommandsReplace => {
            let commands: Vec<Value> = params
                .get("commands")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            crate::core::plugins::commands::replace_commands_inner(state, webview, commands)
                .map_err(|e| PluginRuntimeError::internal(e))?;
            Ok(Value::Null)
        }
        MethodId::CommandsUpdate => {
            let id = required_string_param(&params, "id")?;
            let cmd = params.get("cmd").cloned().unwrap_or(Value::Null);
            crate::core::plugins::commands::update_command_inner(state, webview, &id, &cmd)
                .map_err(|e| PluginRuntimeError::internal(e))?;
            Ok(Value::Null)
        }
        MethodId::SearchRegisterProvider => {
            let provider_id = required_string_param(&params, "id")?;
            let timeout_ms = params
                .get("timeout")
                .and_then(|v| v.as_u64())
                .unwrap_or(300);
            let full_provider_id = format!("{}.{}", context.plugin_id, provider_id);

            // 幂等 replace：先清理旧 provider
            state
                .search_bridge
                .unregister_provider(&full_provider_id);

            let provider: std::sync::Arc<dyn litools_search::SearchProvider> = std::sync::Arc::new(
                crate::core::plugins::runtime::search_provider::WebviewSearchProvider::new(
                    full_provider_id.clone(),
                    webview.label().to_string(),
                    app_handle.clone(),
                    state.search_bridge.clone(),
                    timeout_ms,
                ),
            );

            // 统一通过 bridge 注册（自动写入 SearchEngine + 生命周期元数据）
            state.search_bridge.register_provider(
                crate::core::plugins::runtime::search_bridge::RegisteredSearchProvider {
                    plugin_id: context.plugin_id.clone(),
                    runtime_id: context.id.clone(),
                    provider_id: full_provider_id.clone(),
                    webview_label: webview.label().to_string(),
                    registered_at: chrono::Utc::now().to_rfc3339(),
                },
                provider,
            );

            Ok(json!({ "providerId": full_provider_id }))
        }
        MethodId::SearchUnregisterProvider => {
            let provider_id = required_string_param(&params, "id")?;
            let full_provider_id = format!("{}.{}", context.plugin_id, provider_id);

            // bridge 统一处理 — 同时清理 SearchEngine + 生命周期元数据
            state.search_bridge.unregister_provider(&full_provider_id);
            Ok(Value::Null)
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
        .read().unwrap();
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
