use crate::view::model::{ViewDefinition, ViewKind, ViewProvider, WindowHostKind};

pub const CORE_LAUNCHER_VIEW_ID: &str = "core.launcher";
pub const CORE_SETTINGS_VIEW_ID: &str = "core.settings";
pub const CORE_PLUGINS_VIEW_ID: &str = "core.plugins";
pub const CORE_DIAGNOSTICS_VIEW_ID: &str = "core.diagnostics";

pub fn core_views() -> Vec<ViewDefinition> {
    vec![
        ViewDefinition {
            id: CORE_LAUNCHER_VIEW_ID.to_string(),
            provider: ViewProvider::Core,
            kind: ViewKind::Launcher,
            route: "/".to_string(),
            title: "启动器".to_string(),
            default_host: WindowHostKind::Main,
            allowed_hosts: vec![WindowHostKind::Main],
            detachable: false,
        },
        ViewDefinition {
            id: CORE_SETTINGS_VIEW_ID.to_string(),
            provider: ViewProvider::Core,
            kind: ViewKind::Panel,
            route: "/settings".to_string(),
            title: "设置".to_string(),
            default_host: WindowHostKind::Main,
            allowed_hosts: vec![WindowHostKind::Main, WindowHostKind::Detached],
            detachable: true,
        },
        ViewDefinition {
            id: CORE_PLUGINS_VIEW_ID.to_string(),
            provider: ViewProvider::Core,
            kind: ViewKind::Panel,
            route: "/plugins".to_string(),
            title: "插件中心".to_string(),
            default_host: WindowHostKind::Main,
            allowed_hosts: vec![WindowHostKind::Main, WindowHostKind::Detached],
            detachable: true,
        },
        ViewDefinition {
            id: CORE_DIAGNOSTICS_VIEW_ID.to_string(),
            provider: ViewProvider::Core,
            kind: ViewKind::Panel,
            route: "/diagnostics".to_string(),
            title: "诊断".to_string(),
            default_host: WindowHostKind::Main,
            allowed_hosts: vec![WindowHostKind::Main, WindowHostKind::Detached],
            detachable: true,
        },
    ]
}

pub fn view_for_route(route: &str) -> Option<ViewDefinition> {
    core_views().into_iter().find(|view| view.route == route)
}

#[allow(dead_code)]
pub fn view_for_id(id: &str) -> Option<ViewDefinition> {
    core_views().into_iter().find(|view| view.id == id)
}

pub fn validate_route(route: &str) -> Result<ViewDefinition, String> {
    view_for_route(route).ok_or_else(|| format!("unknown route: {route}"))
}

pub fn validate_host_allowed(
    view: &ViewDefinition,
    host_kind: &WindowHostKind,
) -> Result<(), String> {
    if view.allowed_hosts.contains(host_kind) {
        Ok(())
    } else {
        Err(format!("view cannot be opened in host: {}", view.id))
    }
}

pub fn validate_detachable(view: &ViewDefinition) -> Result<(), String> {
    if view.detachable && view.allowed_hosts.contains(&WindowHostKind::Detached) {
        Ok(())
    } else {
        Err(format!("route cannot be detached: {}", view.route))
    }
}
