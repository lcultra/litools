use std::collections::HashMap;

use chrono::Utc;
use litools_index::{
    IndexDatabase,
    repository::{PluginCommandRepository, PluginCommandUpsert, PluginRepository, PluginUpsert},
};
use litools_plugin::{
    InstalledPlugin, PluginDiscoveryRoot, PluginManager, PluginSource, discover_plugins,
    plugin_result_id,
};
use rusqlite::params;

use crate::{
    app::{AppBootstrapPaths, LitoolsApp},
    error::LitoolsResult,
};

impl LitoolsApp {
    /// 从数据库重新加载 PluginManager 并使搜索缓存失效
    pub fn reload_plugins(&mut self) -> LitoolsResult<()> {
        let manager = load_plugins_from_database(&self.context.database)?;
        self.context.plugins = manager;
        self.plugin_command_provider.invalidate_cache();
        Ok(())
    }

    /// 原子 reload 插件：删除旧的 manifest 命令 → 重新加载 PluginManager
    /// 调用方负责在调用前重新同步磁盘上的插件清单到数据库
    pub fn reload_plugin(&mut self, plugin_id: &str) -> LitoolsResult<()> {
        {
            let connection = self.context.database.connection();
            connection.execute(
                "DELETE FROM plugin_commands WHERE plugin_id = ?1 AND source = 'manifest'",
                params![plugin_id],
            )?;
        }

        self.reload_plugins()?;

        Ok(())
    }

    /// 获取插件的 JSON 摘要
    pub fn plugin_summary(&self, plugin_id: &str) -> Option<serde_json::Value> {
        self.context.plugins.find_plugin(plugin_id).map(|p| {
            serde_json::json!({
                "id": p.manifest.id,
                "name": p.manifest.name,
                "version": p.manifest.version,
                "description": p.manifest.description,
                "author": p.manifest.author,
                "icon": format!("litools-plugin://{}/{}", p.manifest.id, p.manifest.icon),
                "enabled": p.enabled,
                "trusted": p.trusted,
                "source": p.source.as_str(),
                "path": p.path.to_string_lossy(),
                "permissions": p.manifest.permissions,
                "commands": p.manifest.commands.iter().map(|c| {
                    serde_json::json!({
                        "id": c.id,
                        "title": c.title,
                        "subtitle": c.subtitle,
                        "keywords": c.keywords,
                        "mode": c.mode.as_str(),
                    })
                }).collect::<Vec<_>>(),
            })
        })
    }
}

pub(crate) fn sync_and_load_plugins(
    database: &IndexDatabase,
    paths: &AppBootstrapPaths,
) -> LitoolsResult<PluginManager> {
    let mut roots = Vec::new();
    if let Some(bundled_plugins_dir) = &paths.bundled_plugins_dir {
        roots.push(PluginDiscoveryRoot {
            path: bundled_plugins_dir.clone(),
            source: PluginSource::Bundled,
        });
    }
    roots.push(PluginDiscoveryRoot {
        path: paths.data_dir.join("plugins"),
        source: PluginSource::User,
    });

    let discovered_plugins = dedupe_discovered_plugins(discover_plugins(roots));
    let ids = discovered_plugins
        .iter()
        .map(|p| format!("  - {}", p.manifest.id))
        .collect::<Vec<_>>()
        .join("\n");
    log::info!("已加载 {} 个插件:\n{ids}", discovered_plugins.len());
    let updated_at = Utc::now().to_rfc3339();

    {
        let connection = database.connection();
        let plugins = PluginRepository::new(&connection);
        let plugin_commands = PluginCommandRepository::new(&connection);
        let mut seen_by_source: HashMap<&'static str, Vec<String>> = HashMap::new();

        for discovered in discovered_plugins {
            let manifest_json = serde_json::to_string(&discovered.manifest)?;
            let existing = plugins.find_plugin(&discovered.manifest.id)?;
            let installed_at = existing
                .as_ref()
                .map(|plugin| plugin.installed_at.as_str())
                .unwrap_or(&updated_at);
            let enabled = existing
                .as_ref()
                .map(|plugin| plugin.enabled)
                .unwrap_or_else(|| discovered.source.default_enabled());
            let trusted = existing
                .as_ref()
                .map(|plugin| plugin.trusted)
                .unwrap_or_else(|| discovered.source.default_trusted());
            let root_dir = discovered.root_dir.to_string_lossy().to_string();
            let source = discovered.source.as_str();
            seen_by_source
                .entry(source)
                .or_default()
                .push(discovered.manifest.id.clone());

            plugins.upsert_plugin(PluginUpsert {
                id: &discovered.manifest.id,
                name: &discovered.manifest.name,
                version: &discovered.manifest.version,
                path: &root_dir,
                manifest_json: &manifest_json,
                source,
                enabled,
                trusted,
                installed_at,
                updated_at: &updated_at,
            })?;

            let command_upserts = discovered
                .manifest
                .commands
                .iter()
                .map(|command| PluginCommandUpsert {
                    id: plugin_result_id(&discovered.manifest.id, &command.id),
                    plugin_id: discovered.manifest.id.clone(),
                    command_id: command.id.clone(),
                    title: command.title.clone(),
                    subtitle: command.subtitle.clone(),
                    keywords: command.keywords.clone(),
                    mode: command.mode.as_str().to_string(),
                    executor: None,
                    icon: None,
                    script: None,
                    source: "manifest".to_string(),
                    lifecycle: "permanent".to_string(),
                    registrar_runtime_id: None,
                    executor_runtime_id: None,
                    permission_requirements: discovered.manifest.permissions.clone(),
                })
                .collect::<Vec<_>>();
            plugin_commands
                .replace_commands_for_plugin(&discovered.manifest.id, &command_upserts)?;
        }

        for source in [PluginSource::Bundled, PluginSource::User] {
            let seen_ids = seen_by_source.remove(source.as_str()).unwrap_or_default();
            plugins.delete_plugins_not_in_source_ids(source.as_str(), &seen_ids)?;
        }
    }

    load_plugins_from_database(database)
}

fn dedupe_discovered_plugins(
    discovered_plugins: Vec<litools_plugin::DiscoveredPlugin>,
) -> Vec<litools_plugin::DiscoveredPlugin> {
    let mut plugins_by_id: HashMap<String, litools_plugin::DiscoveredPlugin> = HashMap::new();

    for plugin in discovered_plugins {
        let id = plugin.manifest.id.clone();
        match plugins_by_id.get(&id) {
            Some(existing)
                if plugin_source_priority(&existing.source)
                    <= plugin_source_priority(&plugin.source) =>
            {
                log::warn!("插件发现: 忽略重复插件 ID {id}");
            }
            _ => {
                if plugins_by_id.contains_key(&id) {
                    log::info!("插件发现: 用高优先级来源替换重复插件 ID {id}");
                }
                plugins_by_id.insert(id, plugin);
            }
        }
    }

    let mut plugins = plugins_by_id.into_values().collect::<Vec<_>>();
    plugins.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    plugins
}

fn plugin_source_priority(source: &PluginSource) -> u8 {
    match source {
        PluginSource::Bundled => 0,
        PluginSource::User => 1,
    }
}

pub(crate) fn load_plugins_from_database(database: &IndexDatabase) -> LitoolsResult<PluginManager> {
    let connection = database.connection();
    let plugins = PluginRepository::new(&connection)
        .list_plugins()?
        .into_iter()
        .filter_map(|record| {
            let manifest = serde_json::from_str(&record.manifest_json).ok()?;
            Some(InstalledPlugin {
                manifest,
                path: record.path.into(),
                source: PluginSource::from_str(&record.source).unwrap_or(PluginSource::User),
                enabled: record.enabled,
                trusted: record.trusted,
                installed_at: record.installed_at,
                updated_at: record.updated_at,
            })
        })
        .collect();

    Ok(PluginManager::hydrate(plugins))
}

pub fn plugin_route(plugin_id: &str, command_id: &str) -> String {
    format!("/plugin/{plugin_id}/{command_id}")
}
