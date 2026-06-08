use crate::view::model::{ViewDefinition, ViewKind, ViewProvider, WindowHostKind};

pub const CORE_LAUNCHER_VIEW_ID: &str = "core.launcher";

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
    ]
}

pub fn view_for_route(route: &str) -> Option<ViewDefinition> {
    core_views().into_iter().find(|view| view.route == route)
}

pub fn plugin_runtime_route_parts(route: &str) -> Option<(&str, &str)> {
    let rest = route.strip_prefix("/plugin-runtime/")?;
    let (plugin_id, command_id) = rest.split_once('/')?;
    if plugin_id.is_empty() || command_id.is_empty() || command_id.contains('/') {
        return None;
    }
    Some((plugin_id, command_id))
}

pub fn plugin_runtime_view(
    plugin_id: &str,
    command_id: &str,
    title: impl Into<String>,
) -> ViewDefinition {
    ViewDefinition {
        id: format!("plugin.{plugin_id}.{command_id}"),
        provider: ViewProvider::Plugin {
            plugin_id: plugin_id.to_string(),
        },
        kind: ViewKind::Runtime,
        route: format!("/plugin-runtime/{plugin_id}/{command_id}"),
        title: title.into(),
        default_host: WindowHostKind::Runtime,
        allowed_hosts: vec![
            WindowHostKind::Main,
            WindowHostKind::Detached,
            WindowHostKind::Runtime,
        ],
        detachable: true,
    }
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
