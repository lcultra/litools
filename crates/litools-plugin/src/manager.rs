use std::{collections::HashMap, path::PathBuf};

use crate::manifest::{PluginCommand, PluginManifest};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PluginSource {
    Bundled,
    User,
}

impl PluginSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            PluginSource::Bundled => "bundled",
            PluginSource::User => "user",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "bundled" => Some(Self::Bundled),
            "user" => Some(Self::User),
            _ => None,
        }
    }

    pub fn default_enabled(&self) -> bool {
        true
    }

    pub fn default_trusted(&self) -> bool {
        matches!(self, Self::Bundled)
    }
}

#[derive(Default)]
pub struct PluginManager {
    installed_plugins: HashMap<String, InstalledPlugin>,
}

#[derive(Clone, Debug)]
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub source: PluginSource,
    pub enabled: bool,
    pub trusted: bool,
    pub installed_at: String,
    pub updated_at: String,
}

impl PluginManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hydrate(installed_plugins: Vec<InstalledPlugin>) -> Self {
        Self {
            installed_plugins: installed_plugins
                .into_iter()
                .map(|plugin| (plugin.manifest.id.clone(), plugin))
                .collect(),
        }
    }

    pub fn register_local_plugin(&mut self, manifest: PluginManifest, path: PathBuf) {
        let now = String::new();
        self.upsert_plugin(InstalledPlugin {
            manifest,
            path,
            source: PluginSource::User,
            enabled: true,
            trusted: false,
            installed_at: now.clone(),
            updated_at: now,
        });
    }

    pub fn upsert_plugin(&mut self, plugin: InstalledPlugin) {
        self.installed_plugins
            .insert(plugin.manifest.id.clone(), plugin);
    }

    pub fn installed_plugins(&self) -> Vec<&InstalledPlugin> {
        let mut plugins = self.installed_plugins.values().collect::<Vec<_>>();
        plugins.sort_by(|left, right| left.manifest.name.cmp(&right.manifest.name));
        plugins
    }

    pub fn enabled_plugins(&self) -> Vec<&InstalledPlugin> {
        self.installed_plugins()
            .into_iter()
            .filter(|plugin| plugin.enabled)
            .collect()
    }

    pub fn find_plugin(&self, plugin_id: &str) -> Option<&InstalledPlugin> {
        self.installed_plugins.get(plugin_id)
    }

    pub fn find_command(&self, plugin_id: &str, command_id: &str) -> Option<&PluginCommand> {
        self.find_plugin(plugin_id).and_then(|plugin| {
            plugin
                .manifest
                .commands
                .iter()
                .find(|command| command.id == command_id)
        })
    }

    pub fn len(&self) -> usize {
        self.installed_plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.installed_plugins.is_empty()
    }
}
