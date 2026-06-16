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

    /// 返回所有已启用插件的 View 模式命令，用于搜索提供器直接读取，
    /// 无需经过数据库缓存。
    pub fn enabled_view_commands(&self) -> Vec<PluginCommandSearchItem> {
        self.enabled_plugins()
            .into_iter()
            .flat_map(|plugin| {
                let plugin_id = plugin.manifest.id.clone();
                let plugin_name = plugin.manifest.name.clone();
                let plugin_icon = plugin.manifest.icon.clone();
                plugin
                    .manifest
                    .commands
                    .iter()
                    .filter(|c| c.mode == crate::manifest::PluginCommandMode::View)
                    .map(move |command| PluginCommandSearchItem {
                        plugin_id: plugin_id.clone(),
                        plugin_name: plugin_name.clone(),
                        plugin_icon: plugin_icon.clone(),
                        command_id: command.id.clone(),
                        title: command.title.clone(),
                        subtitle: command.subtitle.clone(),
                        keywords: command.keywords.clone(),
                    })
            })
            .collect()
    }
}

/// 插件命令搜索项——从 [`PluginManager`] 直接提取的轻量结构，
/// 避免通过数据库缓存中转。
#[derive(Clone, Debug)]
pub struct PluginCommandSearchItem {
    pub plugin_id: String,
    pub plugin_name: String,
    pub plugin_icon: String,
    pub command_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub keywords: Vec<String>,
}
