// view.rs — 合并自 view/model.rs + view/registry.rs

use serde::Serialize;

// === 原 model.rs ===

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewProvider {
    Core,
    Plugin { plugin_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewKind {
    Launcher,
    Plugin,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowHostKind {
    Main,
    Detached,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewDefinition {
    pub id: String,
    pub provider: ViewProvider,
    pub kind: ViewKind,
    pub route: String,
    pub title: String,
    pub default_host: WindowHostKind,
    pub allowed_hosts: Vec<WindowHostKind>,
    pub detachable: bool,
}

// === 原 registry.rs ===

pub use litools_config::labels::CORE_LAUNCHER_VIEW_ID;

pub fn core_views() -> Vec<ViewDefinition> {
    vec![ViewDefinition {
        id: CORE_LAUNCHER_VIEW_ID.to_string(),
        provider: ViewProvider::Core,
        kind: ViewKind::Launcher,
        route: "/".to_string(),
        title: "启动器".to_string(),
        default_host: WindowHostKind::Main,
        allowed_hosts: vec![WindowHostKind::Main],
        detachable: false,
    }]
}

pub fn view_for_route(route: &str) -> Option<ViewDefinition> {
    core_views().into_iter().find(|view| view.route == route)
}

pub fn plugin_route_parts(route: &str) -> Option<(&str, &str)> {
    let rest = route.strip_prefix("/plugin/")?;
    let (plugin_id, command_id) = rest.split_once('/')?;
    if plugin_id.is_empty() || command_id.is_empty() || command_id.contains('/') {
        return None;
    }
    Some((plugin_id, command_id))
}

pub fn plugin_view_definition(
    plugin_id: &str,
    command_id: &str,
    title: impl Into<String>,
) -> ViewDefinition {
    ViewDefinition {
        id: format!("plugin.{plugin_id}.{command_id}"),
        provider: ViewProvider::Plugin {
            plugin_id: plugin_id.to_string(),
        },
        kind: ViewKind::Plugin,
        route: format!("/plugin/{plugin_id}/{command_id}"),
        title: title.into(),
        default_host: WindowHostKind::Detached,
        allowed_hosts: vec![WindowHostKind::Main, WindowHostKind::Detached],
        detachable: true,
    }
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
