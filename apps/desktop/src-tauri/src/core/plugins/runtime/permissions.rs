use crate::core::plugins::runtime::model::{PermissionQueryState, PluginRuntimeContext};
use crate::core::{CORE_PREFIX, SDK_PREFIX};
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
/// 用于 JS 拦截器阶段的权限检查，当前由 Tauri ACL add_capability 替代。
#[allow(dead_code)]
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

/// 权限域：确定一个 ACL permission identifier 属于哪个信任域。
pub enum PermissionDomain {
    /// litools-core:*，仅宿主主窗口。
    Host,
    /// litools-sdk:*，第三方可按策略授予。
    Sdk,
    /// 非 builtin 的合法 permission identifier（如 `clipboard-manager:default`）。
    /// 是否真实存在由 Tauri ACL 校验。
    Official,
    /// 格式非法（不是合法的 `plugin:name` 格式）。
    Unknown,
}

/// 校验是否为合法的 permission identifier —— 恰好一个 `:`，两侧非空。
fn is_permission_identifier(perm: &str) -> bool {
    // Permission identifier 不含 |
    if perm.contains('|') {
        return false;
    }
    let mut parts = perm.split(':');
    let left = parts.next();
    let right = parts.next();
    left.is_some_and(|l| !l.is_empty())
        && right.is_some_and(|r| !r.is_empty())
        && parts.next().is_none()
}

pub fn categorize_permission(perm: &str) -> PermissionDomain {
    if perm.starts_with(CORE_PREFIX) {
        PermissionDomain::Host
    } else if perm.starts_with(SDK_PREFIX) {
        PermissionDomain::Sdk
    } else if is_permission_identifier(perm) {
        PermissionDomain::Official
    } else {
        PermissionDomain::Unknown
    }
}

/// 插件 manifest 权限声明验证：拒绝 Builtin 权限和格式非法的字符串。
pub fn validate_declared_permissions(perms: &[String]) -> Result<(), String> {
    for perm in perms {
        match categorize_permission(perm) {
            PermissionDomain::Host => {
                return Err(format!("plugin cannot request internal permission: {perm}"));
            }
            PermissionDomain::Unknown => {
                return Err(format!("malformed permission: {perm}"));
            }
            _ => {}
        }
    }
    Ok(())
}

/// 根据插件的 manifest 权限声明和 trusted 状态，构建并注册 capability。
pub fn setup_plugin_capability(
    app: &tauri::AppHandle,
    webview_label: &str,
    declared_permissions: &[String],
    trusted: bool,
) -> Result<(), String> {
    // 前置验证：非法声明直接拒绝
    validate_declared_permissions(declared_permissions)?;

    let cap_id = format!("plugin-cap-{webview_label}");
    let mut builder = CapabilityBuilder::new(cap_id).webview(webview_label);

    for perm in declared_permissions {
        match categorize_permission(perm) {
            PermissionDomain::Host | PermissionDomain::Unknown => {
                continue; // 已在 validate 中拒绝，此处兜底
            }
            PermissionDomain::Sdk => {
                if is_internal_sdk_perm(perm) && !trusted {
                    continue;
                }
                builder = builder.permission(perm);
            }
            PermissionDomain::Official => {
                builder = builder.permission(perm);
            }
        }
    }

    app.add_capability(builder)
        .map_err(|e| format!("failed to add capability: {e}"))
}

/// 判断是否为 litools-sdk 的内部权限（仅 trusted 插件可授予）。
/// 判断是否为 litools-sdk 的内部权限（仅 trusted 插件可授予）。
/// 后续 Phase 4 引入 litools-internal 插件后，此处改为前缀匹配。
fn is_internal_sdk_perm(perm: &str) -> bool {
    perm.starts_with("litools-sdk:allow-diagnostics")
        || perm.starts_with("litools-sdk:allow-plugins")
        || perm.starts_with("litools-sdk:allow-settings-update")
}

#[cfg(test)]
mod tests {
    use crate::core::plugins::runtime::model::{
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

    // -- check_toplevel_invoke tests (reserved for fallback) --

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

    // -- validate_declared_permissions tests --

    fn s(v: &str) -> String {
        v.to_string()
    }

    #[test]
    fn validate_allows_official_plugins() {
        assert!(validate_declared_permissions(&[s("clipboard-manager:default")]).is_ok());
        assert!(validate_declared_permissions(&[s("fs:allow-read-text-file")]).is_ok());
        assert!(validate_declared_permissions(&[s("shell:allow-open")]).is_ok());
        // 格式合法但可能不存在 → 由 Tauri add_capability 校验
        assert!(validate_declared_permissions(&[s("unknown-plugin:allow-foo")]).is_ok());
    }

    #[test]
    fn validate_allows_sdk_permissions() {
        assert!(validate_declared_permissions(&[s("litools-sdk:allow-storage")]).is_ok());
        assert!(validate_declared_permissions(&[s("litools-sdk:allow-runtime")]).is_ok());
        assert!(validate_declared_permissions(&[s("litools-sdk:allow-ui")]).is_ok());
    }

    #[test]
    fn validate_rejects_host() {
        assert!(validate_declared_permissions(&[s("litools-core:allow-search")]).is_err());
        assert!(validate_declared_permissions(&[s("litools-core:allow-window")]).is_err());
        assert!(validate_declared_permissions(&[s("litools-core:allow-settings")]).is_err());
    }

    #[test]
    fn validate_rejects_malformed() {
        assert!(validate_declared_permissions(&[s("plugin:unknown-plugin|foo")]).is_err());
        assert!(validate_declared_permissions(&[s("random-string")]).is_err());
        assert!(validate_declared_permissions(&[s("foo:bar:baz")]).is_err());
        assert!(validate_declared_permissions(&[s(":abc")]).is_err());
        assert!(validate_declared_permissions(&[s("abc:")]).is_err());
    }

    #[test]
    fn validate_rejects_mixed() {
        assert!(validate_declared_permissions(&[
            s("clipboard-manager:default"),
            s("litools-core:allow-search"), // 非法
        ])
        .is_err());
    }
}
