use crate::plugin_runtime::model::{PermissionQueryState, PluginRuntimeContext};

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
            header_webview_label: None,
            webview_label: "plugin-runtime-runtime_000001".to_string(),
            placement: PluginRuntimePlacement::Docked,
            bounds: None,
            permissions: permissions.into_iter().map(str::to_string).collect(),
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
}
