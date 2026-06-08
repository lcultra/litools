use litools_core::plugin_runtime_route;
use serde::Serialize;
use tauri::State;

use crate::{plugin_protocol::plugin_asset_url, state::AppState};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCommandSummary {
    id: String,
    title: String,
    subtitle: Option<String>,
    keywords: Vec<String>,
    mode: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSummary {
    id: String,
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
    icon: String,
    enabled: bool,
    trusted: bool,
    source: String,
    path: String,
    permissions: Vec<String>,
    commands: Vec<PluginCommandSummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRuntimeDescriptor {
    plugin_id: String,
    command_id: String,
    plugin_name: String,
    title: String,
    entry_url: String,
    permissions: Vec<String>,
}

#[tauri::command]
pub fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginSummary>, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    Ok(app
        .context()
        .plugins
        .installed_plugins()
        .into_iter()
        .map(|plugin| PluginSummary {
            id: plugin.manifest.id.clone(),
            name: plugin.manifest.name.clone(),
            version: plugin.manifest.version.clone(),
            description: plugin.manifest.description.clone(),
            author: plugin.manifest.author.clone(),
            icon: plugin.manifest.icon.clone(),
            enabled: plugin.enabled,
            trusted: plugin.trusted,
            source: plugin.source.as_str().to_string(),
            path: plugin.path.display().to_string(),
            permissions: plugin.manifest.permissions.clone(),
            commands: plugin
                .manifest
                .commands
                .iter()
                .map(|command| PluginCommandSummary {
                    id: command.id.clone(),
                    title: command.title.clone(),
                    subtitle: command.subtitle.clone(),
                    keywords: command.keywords.clone(),
                    mode: command.mode.as_str().to_string(),
                })
                .collect(),
        })
        .collect())
}

#[tauri::command]
pub fn get_plugin_runtime_descriptor(
    plugin_id: String,
    command_id: String,
    state: State<'_, AppState>,
) -> Result<PluginRuntimeDescriptor, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    let plugin = app
        .context()
        .plugins
        .find_plugin(&plugin_id)
        .ok_or_else(|| format!("plugin not found: {plugin_id}"))?;
    if !plugin.enabled {
        return Err(format!("plugin is disabled: {plugin_id}"));
    }
    let command = plugin
        .manifest
        .commands
        .iter()
        .find(|command| command.id == command_id)
        .ok_or_else(|| format!("plugin command not found: {plugin_id}:{command_id}"))?;

    let entry_url = plugin_entry_url(&plugin.manifest.id, &plugin.manifest.entry)?;

    Ok(PluginRuntimeDescriptor {
        plugin_id,
        command_id,
        plugin_name: plugin.manifest.name.clone(),
        title: command.title.clone(),
        entry_url,
        permissions: plugin.manifest.permissions.clone(),
    })
}

pub fn validate_plugin_runtime_route(
    state: &AppState,
    route: &str,
) -> Result<crate::view::model::ViewDefinition, String> {
    let Some((plugin_id, command_id)) = crate::view::registry::plugin_runtime_route_parts(route)
    else {
        return Err(format!("unknown route: {route}"));
    };

    let app = state.app().lock().map_err(|error| error.to_string())?;
    let plugin = app
        .context()
        .plugins
        .find_plugin(plugin_id)
        .ok_or_else(|| format!("unknown plugin route: {route}"))?;
    if !plugin.enabled {
        return Err(format!("plugin is disabled: {plugin_id}"));
    }
    let command = plugin
        .manifest
        .commands
        .iter()
        .find(|command| command.id == command_id)
        .ok_or_else(|| format!("unknown plugin command route: {route}"))?;

    Ok(crate::view::registry::plugin_runtime_view(
        plugin_id,
        command_id,
        command.title.clone(),
    ))
}

fn plugin_entry_url(plugin_id: &str, entry: &str) -> Result<String, String> {
    let entry_path = std::path::Path::new(entry);
    if entry_path.is_absolute()
        || entry_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!("invalid plugin entry: {entry}"));
    }

    Ok(plugin_asset_url(plugin_id, entry))
}

#[allow(dead_code)]
fn _route_for_descriptor(plugin_id: &str, command_id: &str) -> String {
    plugin_runtime_route(plugin_id, command_id)
}
