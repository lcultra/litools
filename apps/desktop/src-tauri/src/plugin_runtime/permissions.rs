use crate::plugin_runtime::model::{PermissionQueryState, PluginRuntimeContext};
use tauri::ipc::CapabilityBuilder;
use tauri::Manager;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePermissionRequirement {
    Intrinsic,
    Permission(&'static str),
}

pub fn required_permission_for_method(method: &str) -> Option<RuntimePermissionRequirement> {
    match method {
        "runtime.ready" | "runtime.getInfo" | "permissions.query" => {
            Some(RuntimePermissionRequirement::Intrinsic)
        }
        "ui.close" | "ui.setTitle" => Some(RuntimePermissionRequirement::Permission("ui:window")),
        "ui.toast" => Some(RuntimePermissionRequirement::Permission("ui:toast")),
        "storage.get" | "storage.set" | "storage.remove" | "storage.clear" => {
            Some(RuntimePermissionRequirement::Permission("storage:plugin"))
        }
        "settings.get" => Some(RuntimePermissionRequirement::Permission("settings:read")),
        "settings.update" => Some(RuntimePermissionRequirement::Permission("settings:write")),
        "diagnostics.get" => Some(RuntimePermissionRequirement::Permission("diagnostics:read")),
        "plugins.list" => Some(RuntimePermissionRequirement::Permission("plugins:list")),
        _ => None,
    }
}

pub fn is_permission_granted(context: &PluginRuntimeContext, permission: &str) -> bool {
    context
        .permissions
        .iter()
        .any(|declared| declared == permission)
}

pub fn can_call_method(context: &PluginRuntimeContext, method: &str) -> bool {
    match required_permission_for_method(method) {
        Some(RuntimePermissionRequirement::Intrinsic) => true,
        Some(RuntimePermissionRequirement::Permission(permission)) => {
            is_permission_granted(context, permission)
        }
        None => false,
    }
}

pub fn query_permission(context: &PluginRuntimeContext, permission: &str) -> PermissionQueryState {
    if is_permission_granted(context, permission) {
        PermissionQueryState::Granted
    } else {
        PermissionQueryState::Denied
    }
}

/// 检查插件 webview 发出的顶层 invoke('plugin:xxx|yyy') 是否有权限。
///
/// `command` 格式: "plugin:{plugin_name}|{action}"
pub fn check_toplevel_invoke(context: &PluginRuntimeContext, command: &str) -> bool {
    // 仅处理 plugin:* 命令，其他的直接放行
    let rest = match command.strip_prefix("plugin:") {
        Some(r) => r,
        None => return true,
    };

    // 解析 plugin_name 和 action
    let (plugin_name, action) = match rest.split_once('|') {
        Some((name, act)) => (name, act),
        None => return false,
    };

    // 精确匹配: "{plugin_name}:allow-{action}"
    let exact = format!("{plugin_name}:allow-{action}");
    if context.permissions.iter().any(|p| p == &exact) {
        return true;
    }

    // default 匹配: "{plugin_name}:default"（Phase 1 简化：default 覆盖该插件全部命令）
    let default = format!("{plugin_name}:default");
    if context.permissions.iter().any(|p| p == &default) {
        return true;
    }

    false
}

/// 命令分类。
pub enum CommandCategory {
    Host,
    Internal,
    Sdk,
}

/// 根据 IPC 方法名返回命令分类。将来拆 crate 后用权限前缀判断，现阶段用显式列表。
pub fn categorize_method(method: &str) -> CommandCategory {
    match method {
        // Host — 启动器控制面，仅主窗口
        "search" | "launcher_panel" | "pin_result" | "unpin_result"
        | "reorder_pinned_results" | "execute_result"
        | "hide_main_window" | "show_main_window" | "focus_main_window"
        | "resize_main_window_height" | "start_window_dragging"
        | "hide_window" | "focus_window" | "destroy_window"
        | "list_windows" | "get_current_window_metadata"
        | "detach_route" | "update_surface_route"
        | "reveal_in_file_manager"
        | "reload_index" | "get_diagnostics"
        | "get_settings" | "update_settings"
        | "list_plugins" | "get_plugin_view_descriptor"
        | "open_plugin_view" | "hide_plugin_view" | "detach_plugin_view"
        | "close_plugin_view" | "close_plugin_view_by_id"
        | "get_plugin_view_info" | "open_plugin_devtools" => CommandCategory::Host,

        // Internal — 仅 trusted 插件
        "plugins.list" | "diagnostics.get" | "settings.update" => CommandCategory::Internal,

        // Sdk — 公开，按 plugin.json 声明决定是否授予
        _ => CommandCategory::Sdk,
    }
}

/// 根据插件的 manifest 权限声明和 trusted 状态，构建并注册 capability。
pub fn setup_plugin_capability(
    app: &tauri::AppHandle,
    webview_label: &str,
    declared_permissions: &[String],
    trusted: bool,
) -> Result<(), String> {
    let cap_id = format!("plugin-cap-{webview_label}");
    let mut builder = CapabilityBuilder::new(cap_id).webview(webview_label);

    for perm in declared_permissions {
        // 内部权限：仅 trusted 插件
        if perm.starts_with("litools-sdk:allow-diagnostics")
            || perm.starts_with("litools-sdk:allow-plugins")
            || perm.starts_with("litools-sdk:allow-settings-update")
            || perm.starts_with("litools-internal:")
        {
            if !trusted {
                continue;
            }
        }
        // Host 权限：绝不给插件
        if perm.starts_with("litools-host:") {
            continue;
        }
        builder = builder.permission(perm);
    }

    app.add_capability(builder)
        .map_err(|e| format!("failed to add capability: {e}"))
}

#[cfg(test)]
mod tests {
    use crate::plugin_runtime::model::{
        PluginRuntimeContext, PluginRuntimeLifecycle, PluginRuntimePlacement,
    };

    use super::*;

    fn context(permissions: Vec<&str>) -> PluginRuntimeContext {
        PluginRuntimeContext {
            id: "runtime_000001".to_string(),
            plugin_id: "dev.litools.test".to_string(),
            command_id: "hello".to_string(),
            plugin_name: "Test".to_string(),
            title: "Hello".to_string(),
            entry_url: "litools-plugin://dev.litools.test/dist/index.html".to_string(),
            host_window_label: "main".to_string(),
            detached_window_label: None,
            webview_label: "plugin-runtime-runtime_000001".to_string(),
            placement: PluginRuntimePlacement::Docked,
            bounds: None,
            permissions: permissions.into_iter().map(str::to_string).collect(),
            trusted: false,
            policy: litools_plugin::RuntimePolicy::Singleton,
            lifecycle: PluginRuntimeLifecycle::Created,
            pending_enter: false,
            entered: false,
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        }
    }

    #[test]
    fn allows_intrinsic_methods() {
        assert!(can_call_method(&context(vec![]), "runtime.ready"));
        assert!(can_call_method(&context(vec![]), "runtime.getInfo"));
        assert!(can_call_method(&context(vec![]), "permissions.query"));
    }

    #[test]
    fn denies_unknown_methods() {
        assert!(!can_call_method(
            &context(vec!["ui:window"]),
            "window.invoke"
        ));
    }

    #[test]
    fn requires_declared_permissions() {
        assert!(!can_call_method(&context(vec![]), "ui.close"));
        assert!(can_call_method(&context(vec!["ui:window"]), "ui.close"));
        assert!(!can_call_method(&context(vec!["ui:window"]), "storage.get"));
        assert!(can_call_method(
            &context(vec!["storage:plugin"]),
            "storage.get"
        ));
    }

    #[test]
    fn toplevel_allow_exact_match() {
        let ctx = context(vec!["clipboard-manager:allow-write_text"]);
        assert!(check_toplevel_invoke(&ctx, "plugin:clipboard-manager|write_text"));
    }

    #[test]
    fn toplevel_allow_default() {
        let ctx = context(vec!["clipboard-manager:default"]);
        assert!(check_toplevel_invoke(&ctx, "plugin:clipboard-manager|write_text"));
    }

    #[test]
    fn toplevel_deny_undeclared() {
        let ctx = context(vec!["clipboard-manager:allow-read_text"]);
        assert!(!check_toplevel_invoke(&ctx, "plugin:clipboard-manager|write_text"));
    }

    #[test]
    fn toplevel_passthrough_non_plugin() {
        let ctx = context(vec![]);
        assert!(check_toplevel_invoke(&ctx, "core:window|close"));
    }

    #[test]
    fn toplevel_deny_malformed() {
        let ctx = context(vec!["anything:default"]);
        assert!(!check_toplevel_invoke(&ctx, "plugin:no_pipe"));
    }
}
