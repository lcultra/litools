use serde::Serialize;
use tauri::State;

use crate::{
    plugin_runtime::service::find_enabled_plugin_command, protocol::plugin::plugin_entry_url,
    state::AppState,
};

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
pub struct PluginViewDescriptor {
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
pub fn get_plugin_view_descriptor(
    plugin_id: String,
    command_id: String,
    state: State<'_, AppState>,
) -> Result<PluginViewDescriptor, String> {
    let (plugin_name, title, permissions, _policy) =
        find_enabled_plugin_command(&state, &plugin_id, &command_id)?;

    let app = state.app().lock().map_err(|error| error.to_string())?;
    let plugin = app.context().plugins.find_plugin(&plugin_id).unwrap();
    let entry_url = plugin_entry_url(&plugin.manifest.id, &plugin.manifest.entry)?;

    Ok(PluginViewDescriptor {
        plugin_id,
        command_id,
        plugin_name,
        title,
        entry_url,
        permissions,
    })
}

pub fn validate_plugin_view_route(
    state: &AppState,
    route: &str,
) -> Result<crate::view::model::ViewDefinition, String> {
    let Some((plugin_id, command_id)) = crate::view::registry::plugin_route_parts(route) else {
        return Err(format!("unknown route: {route}"));
    };

    let (_, title, _, _) = find_enabled_plugin_command(state, plugin_id, command_id)
        .map_err(|_| format!("unknown plugin route: {route}"))?;

    Ok(crate::view::registry::plugin_view_definition(
        plugin_id, command_id, title,
    ))
}
