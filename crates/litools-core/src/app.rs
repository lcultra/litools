use std::{collections::HashMap, path::PathBuf, sync::Arc};

use chrono::{DateTime, Utc};
use litools_index::{
    IndexDatabase,
    repository::{
        AppRecord, AppRepository, AppUpsert, CommandRepository, IndexMetadataRepository,
        PinnedRepository, PluginCommandRepository, PluginCommandUpsert, PluginRepository,
        PluginUpsert, SettingsRepository, UsageEventRecord, UsageRepository,
    },
};
use litools_plugin::{
    InstalledPlugin, PluginCommandMode, PluginDiscoveryRoot, PluginManager, PluginSource,
    discover_plugins,
};
use litools_search::{SearchEngine, SearchQuery, SearchResult};
use litools_settings::{AppSettings, storage::SettingsStore};
use litools_system::{NativeSystemAdapter, SystemAdapter};
use litools_telemetry::init_logging;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const APP_SETTINGS_KEY: &str = "app_settings";
const APPS_INDEX_STATUS_KEY: &str = "apps_last_refresh_status";
const LAUNCHER_RESULT_LIMIT: usize = 20;
const RELOAD_INDEX_TRIGGER_DIRECT: &str = "direct";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReloadIndexSummary {
    pub trigger: String,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: i64,
    pub commands_upserted: usize,
    pub apps_discovered: usize,
    pub apps_upserted: usize,
    pub apps_removed: usize,
    pub success: bool,
    pub error: Option<String>,
}

use crate::{
    app_provider::{
        AppSearchProvider, OPEN_APP_ACTION_ID, app_id_from_result_id, search_result_for_app,
    },
    command::{
        BUILTIN_COMMANDS, BuiltinCommandProvider, CommandEffect, CommandExecution,
        find_builtin_command, search_result_for_builtin_command,
    },
    context::AppContext,
    error::{LitoolsError, LitoolsResult},
    launcher::{LauncherItem, LauncherPanelResponse, LauncherSection},
    plugin_provider::{
        OPEN_PLUGIN_ACTION_ID, PLUGIN_TARGET_TYPE, PluginCommandProvider,
        plugin_command_from_result_id, plugin_command_from_target_id, plugin_result_id,
        plugin_target_id, search_result_for_plugin_command_record,
    },
};

pub struct AppBootstrapPaths {
    pub data_dir: PathBuf,
    pub bundled_plugins_dir: Option<PathBuf>,
}

impl AppBootstrapPaths {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            bundled_plugins_dir: None,
        }
    }
}

pub struct LitoolsApp {
    context: AppContext,
    paths: AppBootstrapPaths,
}

impl LitoolsApp {
    pub fn bootstrap(paths: AppBootstrapPaths) -> LitoolsResult<Self> {
        init_logging();

        std::fs::create_dir_all(&paths.data_dir)?;
        let database = IndexDatabase::open(paths.data_dir.join("database.sqlite"))?;
        let settings = load_settings(&database)?;
        let plugins = sync_and_load_plugins(&database, &paths)?;
        let search = default_search_engine(database.clone());

        Ok(Self {
            context: AppContext::new(database, search, plugins, SettingsStore::new(settings)),
            paths,
        })
    }

    pub fn bootstrap_in_memory() -> LitoolsResult<Self> {
        init_logging();

        let database = IndexDatabase::in_memory()?;
        let settings = load_settings(&database)?;
        let plugins = load_plugins_from_database(&database)?;
        let search = default_search_engine(database.clone());

        Ok(Self {
            context: AppContext::new(database, search, plugins, SettingsStore::new(settings)),
            paths: AppBootstrapPaths::new(""),
        })
    }

    pub fn search(&self, text: impl Into<String>) -> Vec<SearchResult> {
        let settings = self.context.settings.get();
        self.context.search.search_with_providers(
            SearchQuery::with_limit(text, LAUNCHER_RESULT_LIMIT),
            settings.search.enabled_providers.iter().map(String::as_str),
        )
    }

    fn search_without_limit(&self, text: impl Into<String>) -> Vec<SearchResult> {
        let settings = self.context.settings.get();
        self.context.search.search_with_providers(
            SearchQuery::without_limit(text),
            settings.search.enabled_providers.iter().map(String::as_str),
        )
    }

    pub fn settings(&self) -> &AppSettings {
        self.context.settings.get()
    }

    pub fn launcher_panel(&self, query: impl Into<String>) -> LitoolsResult<LauncherPanelResponse> {
        let query = query.into();
        let trimmed_query = query.trim();

        if !trimmed_query.is_empty() {
            let mut items = Vec::new();

            for result in self.search_without_limit(trimmed_query) {
                let is_pinned = match self.target_from_result_id(&result.id) {
                    Some((target_type, target_id)) => {
                        self.is_target_pinned(target_type, &target_id)?
                    }
                    None => false,
                };
                items.push(LauncherItem { result, is_pinned });
            }

            return Ok(LauncherPanelResponse {
                sections: section_if_not_empty("best", "最佳搜索结果", items)
                    .into_iter()
                    .collect(),
            });
        }

        let settings = self.context.settings.get();
        let mut sections = Vec::new();

        if settings.palette.show_recent {
            let recent_items = self.recent_launcher_items(LAUNCHER_RESULT_LIMIT)?;

            if let Some(section) = section_if_not_empty("recent", "最近使用", recent_items) {
                sections.push(section);
            }
        }

        if settings.palette.show_pinned {
            let pinned_items = self.pinned_launcher_items(LAUNCHER_RESULT_LIMIT)?;

            if let Some(section) = section_if_not_empty("pinned", "已固定", pinned_items) {
                sections.push(section);
            }
        }

        Ok(LauncherPanelResponse { sections })
    }

    pub fn pin_result(&self, result_id: impl Into<String>) -> LitoolsResult<()> {
        let result_id = result_id.into();
        let (target_type, target_id) = self.validated_target_from_result_id(&result_id)?;
        let connection = self.context.database.connection();
        PinnedRepository::new(&connection).pin(
            target_type,
            &target_id,
            &Utc::now().to_rfc3339(),
        )?;
        Ok(())
    }

    pub fn unpin_result(&self, result_id: impl Into<String>) -> LitoolsResult<()> {
        let result_id = result_id.into();
        let (target_type, target_id) = self.validated_target_from_result_id(&result_id)?;
        let connection = self.context.database.connection();
        PinnedRepository::new(&connection).unpin(target_type, &target_id)?;
        Ok(())
    }

    pub fn reorder_pinned_results(&self, result_ids: Vec<String>) -> LitoolsResult<()> {
        let mut targets = Vec::with_capacity(result_ids.len());

        for result_id in result_ids {
            let (target_type, target_id) = self.validated_target_from_result_id(&result_id)?;
            targets.push((target_type.to_string(), target_id, result_id));
        }

        let connection = self.context.database.connection();
        let pinned = PinnedRepository::new(&connection);
        let mut ordered_targets = Vec::with_capacity(targets.len());

        for (target_type, target_id, result_id) in targets {
            if !pinned.is_pinned(&target_type, &target_id)? {
                return Err(LitoolsError::CommandNotFound(result_id));
            }

            ordered_targets.push((target_type, target_id));
        }

        pinned.reorder(&ordered_targets)?;
        Ok(())
    }

    pub fn update_settings(&mut self, settings: AppSettings) -> LitoolsResult<AppSettings> {
        let settings = settings.normalized();
        persist_settings(&self.context.database, &settings)?;
        self.context.settings.replace(settings.clone());
        Ok(settings)
    }

    pub fn execute_result(
        &mut self,
        result_id: impl Into<String>,
        action_id: impl Into<String>,
    ) -> LitoolsResult<CommandExecution> {
        let result_id = result_id.into();
        let action_id = action_id.into();

        if let Some(app_id) = app_id_from_result_id(&result_id) {
            return self.execute_app_result(&result_id, app_id, &action_id);
        }

        if let Some((plugin_id, command_id)) = plugin_command_from_result_id(&result_id) {
            return self
                .execute_plugin_command_result(&result_id, plugin_id, command_id, &action_id);
        }

        let effect = builtin_effect_for_result(&result_id)?;

        match effect {
            CommandEffect::ReloadIndex => {
                let _ = self.reload_index()?;
            }
            CommandEffect::ToggleTheme => self.toggle_theme()?,
            _ => {}
        }

        let connection = self.context.database.connection();
        UsageRepository::new(&connection).record_selection(
            &Uuid::new_v4().to_string(),
            "command",
            &result_id,
            None,
            &Utc::now().to_rfc3339(),
        )?;

        Ok(CommandExecution {
            message: message_for_effect(&effect).to_string(),
            result_id,
            action_id,
            effect,
        })
    }

    pub fn reload_index(&mut self) -> LitoolsResult<ReloadIndexSummary> {
        self.reload_index_with_trigger(RELOAD_INDEX_TRIGGER_DIRECT)
    }

    pub fn reload_index_with_trigger(&mut self, trigger: &str) -> LitoolsResult<ReloadIndexSummary> {
        let started_at = Utc::now();
        let discovered_apps = NativeSystemAdapter.discover_apps();
        let apps_discovered = discovered_apps.len();

        // Re-sync plugins from disk so newly added/removed plugins are reflected
        // without requiring a full app restart.
        self.context.plugins =
            sync_and_load_plugins(&self.context.database, &self.paths)?;

        let connection = self.context.database.connection();
        // Drop the database lock before writing to avoid deadlock with the mutable
        // borrow needed to update context.plugins.
        let transaction = connection.unchecked_transaction()?;
        let commands = CommandRepository::new(&transaction);

        for command in BUILTIN_COMMANDS {
            commands.upsert_command(
                command.id,
                "builtin",
                command.title,
                Some(command.subtitle),
                "execute",
            )?;
        }

        let apps = AppRepository::new(&transaction);
        let last_seen_at = started_at.to_rfc3339();
        for app in &discovered_apps {
            apps.upsert_app(AppUpsert {
                id: &app.id,
                name: &app.name,
                path: &app.path,
                icon_path: app.icon_path.as_deref(),
                localized_names: &app.localized_names,
                aliases: &app.aliases,
                search_text: &app.search_text,
                platform: std::env::consts::OS,
                last_seen_at: &last_seen_at,
            })?;
        }

        let apps_removed = apps.delete_apps_not_seen_at(std::env::consts::OS, &last_seen_at)?;
        let finished_at = Utc::now();
        let summary = reload_index_summary(
            trigger,
            started_at,
            finished_at,
            BUILTIN_COMMANDS.len(),
            apps_discovered,
            discovered_apps.len(),
            apps_removed,
            None,
        );
        IndexMetadataRepository::new(&transaction).set_json(
            APPS_INDEX_STATUS_KEY,
            &serde_json::to_string(&summary)?,
            &finished_at.to_rfc3339(),
        )?;
        transaction.commit()?;

        Ok(summary)
    }

    pub fn recent_usage_events(&self, limit: usize) -> LitoolsResult<Vec<UsageEventRecord>> {
        let connection = self.context.database.connection();
        Ok(UsageRepository::new(&connection).recent_events(limit)?)
    }

    pub fn find_app(&self, id: &str) -> LitoolsResult<Option<AppRecord>> {
        let connection = self.context.database.connection();
        Ok(AppRepository::new(&connection).find_app(id)?)
    }

    pub fn command_count(&self) -> LitoolsResult<usize> {
        let connection = self.context.database.connection();
        Ok(CommandRepository::new(&connection).count_commands()?)
    }

    pub fn app_count(&self) -> LitoolsResult<usize> {
        let connection = self.context.database.connection();
        Ok(AppRepository::new(&connection).count_apps()?)
    }

    pub fn index_status(&self) -> LitoolsResult<Option<ReloadIndexSummary>> {
        let connection = self.context.database.connection();
        let Some(metadata) =
            IndexMetadataRepository::new(&connection).get(APPS_INDEX_STATUS_KEY)?
        else {
            return Ok(None);
        };

        Ok(Some(serde_json::from_str(&metadata.value_json)?))
    }

    pub fn usage_event_count(&self) -> LitoolsResult<usize> {
        let connection = self.context.database.connection();
        Ok(UsageRepository::new(&connection).count_events()?)
    }

    fn pinned_launcher_items(&self, limit: usize) -> LitoolsResult<Vec<LauncherItem>> {
        let connection = self.context.database.connection();
        let pinned = PinnedRepository::new(&connection).list_pinned(limit)?;
        let apps = AppRepository::new(&connection);
        let plugin_commands = PluginCommandRepository::new(&connection);
        let mut items = Vec::new();

        for record in pinned {
            if let Some(result) = result_for_target(
                &apps,
                &plugin_commands,
                &record.target_type,
                &record.target_id,
            )? {
                items.push(LauncherItem {
                    result,
                    is_pinned: true,
                });
            }
        }

        Ok(items)
    }

    fn recent_launcher_items(&self, limit: usize) -> LitoolsResult<Vec<LauncherItem>> {
        let connection = self.context.database.connection();
        let usage = UsageRepository::new(&connection).recent_unique_targets(limit)?;
        let apps = AppRepository::new(&connection);
        let plugin_commands = PluginCommandRepository::new(&connection);
        let pinned = PinnedRepository::new(&connection);
        let mut items = Vec::new();

        for record in usage {
            let Some(result) = result_for_target(
                &apps,
                &plugin_commands,
                &record.target_type,
                &record.target_id,
            )?
            else {
                continue;
            };
            let is_pinned = pinned.is_pinned(&record.target_type, &record.target_id)?;
            items.push(LauncherItem { result, is_pinned });

            if items.len() >= limit {
                break;
            }
        }

        Ok(items)
    }

    fn is_target_pinned(&self, target_type: &str, target_id: &str) -> LitoolsResult<bool> {
        let connection = self.context.database.connection();
        Ok(PinnedRepository::new(&connection).is_pinned(target_type, target_id)?)
    }

    fn validated_target_from_result_id(
        &self,
        result_id: &str,
    ) -> LitoolsResult<(&'static str, String)> {
        let Some((target_type, target_id)) = self.target_from_result_id(result_id) else {
            return Err(LitoolsError::CommandNotFound(result_id.to_string()));
        };

        if target_id.is_empty() {
            return Err(LitoolsError::CommandNotFound(result_id.to_string()));
        }

        match target_type {
            "app" => {
                let connection = self.context.database.connection();
                AppRepository::new(&connection)
                    .find_app(&target_id)?
                    .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;
            }
            "command" => {
                find_builtin_command(&target_id)
                    .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;
            }
            PLUGIN_TARGET_TYPE => {
                let Some((plugin_id, command_id)) = plugin_command_from_target_id(&target_id)
                else {
                    return Err(LitoolsError::CommandNotFound(result_id.to_string()));
                };
                let connection = self.context.database.connection();
                PluginCommandRepository::new(&connection)
                    .find_plugin_command(plugin_id, command_id)?
                    .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;
            }
            _ => return Err(LitoolsError::CommandNotFound(result_id.to_string())),
        }

        Ok((target_type, target_id))
    }

    fn target_from_result_id(&self, result_id: &str) -> Option<(&'static str, String)> {
        if let Some(app_id) = app_id_from_result_id(result_id) {
            return Some(("app", app_id.to_string()));
        }

        if let Some((plugin_id, command_id)) = plugin_command_from_result_id(result_id) {
            return Some((PLUGIN_TARGET_TYPE, plugin_target_id(plugin_id, command_id)));
        }

        find_builtin_command(result_id).map(|command| ("command", command.id.to_string()))
    }

    fn execute_plugin_command_result(
        &self,
        result_id: &str,
        plugin_id: &str,
        command_id: &str,
        action_id: &str,
    ) -> LitoolsResult<CommandExecution> {
        if action_id != OPEN_PLUGIN_ACTION_ID {
            return Err(LitoolsError::CommandNotFound(format!(
                "{result_id}:{action_id}"
            )));
        }

        let plugin = self
            .context
            .plugins
            .find_plugin(plugin_id)
            .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;
        if !plugin.enabled {
            return Err(LitoolsError::CommandNotFound(result_id.to_string()));
        }
        let command = plugin
            .manifest
            .commands
            .iter()
            .find(|command| command.id == command_id)
            .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;
        if command.mode != PluginCommandMode::View {
            return Err(LitoolsError::CommandNotFound(result_id.to_string()));
        }

        let route = plugin_runtime_route(plugin_id, command_id);
        let connection = self.context.database.connection();
        UsageRepository::new(&connection).record_selection(
            &Uuid::new_v4().to_string(),
            PLUGIN_TARGET_TYPE,
            &plugin_target_id(plugin_id, command_id),
            None,
            &Utc::now().to_rfc3339(),
        )?;

        Ok(CommandExecution {
            message: format!("正在打开 {}", command.title),
            result_id: result_id.to_string(),
            action_id: action_id.to_string(),
            effect: CommandEffect::OpenPluginView {
                plugin_id: plugin_id.to_string(),
                command_id: command_id.to_string(),
                route,
            },
        })
    }

    fn execute_app_result(
        &self,
        result_id: &str,
        app_id: &str,
        action_id: &str,
    ) -> LitoolsResult<CommandExecution> {
        if action_id != OPEN_APP_ACTION_ID {
            return Err(LitoolsError::CommandNotFound(format!(
                "{result_id}:{action_id}"
            )));
        }

        let connection = self.context.database.connection();
        let apps = AppRepository::new(&connection);
        let app = apps
            .find_app(app_id)?
            .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;

        NativeSystemAdapter
            .launch_app(&app.id)
            .or_else(|_| NativeSystemAdapter.launch_app(&app.path))
            .map_err(LitoolsError::System)?;
        apps.increment_launch_count(&app.id)?;
        UsageRepository::new(&connection).record_selection(
            &Uuid::new_v4().to_string(),
            "app",
            &app.id,
            None,
            &Utc::now().to_rfc3339(),
        )?;

        Ok(CommandExecution {
            message: format!("正在打开 {}", app.name),
            result_id: result_id.to_string(),
            action_id: action_id.to_string(),
            effect: CommandEffect::None,
        })
    }

    fn toggle_theme(&mut self) -> LitoolsResult<()> {
        let mut settings = self.context.settings.get().clone();
        settings.theme = match settings.theme.as_str() {
            "dark" => "light".to_string(),
            "light" => "dark".to_string(),
            _ => "dark".to_string(),
        };
        self.update_settings(settings)?;
        Ok(())
    }

    pub fn context(&self) -> &AppContext {
        &self.context
    }
}

fn section_if_not_empty(
    id: impl Into<String>,
    title: impl Into<String>,
    items: Vec<LauncherItem>,
) -> Option<LauncherSection> {
    if items.is_empty() {
        return None;
    }

    Some(LauncherSection {
        id: id.into(),
        title: title.into(),
        items,
    })
}

fn result_for_target(
    apps: &AppRepository<'_>,
    plugin_commands: &PluginCommandRepository<'_>,
    target_type: &str,
    target_id: &str,
) -> LitoolsResult<Option<SearchResult>> {
    match target_type {
        "app" => Ok(apps
            .find_app(target_id)?
            .map(|app| search_result_for_app(app, ""))),
        "command" => Ok(find_builtin_command(target_id)
            .map(|command| search_result_for_builtin_command(command, ""))),
        PLUGIN_TARGET_TYPE => {
            let Some((plugin_id, command_id)) = plugin_command_from_target_id(target_id) else {
                return Ok(None);
            };
            Ok(plugin_commands
                .find_plugin_command(plugin_id, command_id)?
                .map(|command| search_result_for_plugin_command_record(command, "")))
        }
        _ => Ok(None),
    }
}

fn reload_index_summary(
    trigger: &str,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    commands_upserted: usize,
    apps_discovered: usize,
    apps_upserted: usize,
    apps_removed: usize,
    error: Option<String>,
) -> ReloadIndexSummary {
    ReloadIndexSummary {
        trigger: trigger.to_string(),
        started_at: started_at.to_rfc3339(),
        finished_at: finished_at.to_rfc3339(),
        duration_ms: (finished_at - started_at).num_milliseconds(),
        commands_upserted,
        apps_discovered,
        apps_upserted,
        apps_removed,
        success: error.is_none(),
        error,
    }
}

fn default_search_engine(database: IndexDatabase) -> SearchEngine {
    let mut search = SearchEngine::new();
    search.register_provider(Arc::new(BuiltinCommandProvider));
    search.register_provider(Arc::new(AppSearchProvider::new(database.clone())));
    search.register_provider(Arc::new(PluginCommandProvider::new(database)));
    search
}

fn sync_and_load_plugins(
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
    eprintln!(
        "sync_and_load_plugins: discovered {} plugin(s): {:?}",
        discovered_plugins.len(),
        discovered_plugins
            .iter()
            .map(|p| &p.manifest.id)
            .collect::<Vec<_>>()
    );
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
                eprintln!("plugin discovery: ignoring duplicate plugin id {id}");
            }
            _ => {
                if plugins_by_id.contains_key(&id) {
                    eprintln!(
                        "plugin discovery: replacing duplicate plugin id {id} with higher-priority source"
                    );
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

fn load_plugins_from_database(database: &IndexDatabase) -> LitoolsResult<PluginManager> {
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

pub fn plugin_runtime_route(plugin_id: &str, command_id: &str) -> String {
    format!("/plugin-runtime/{plugin_id}/{command_id}")
}

fn load_settings(database: &IndexDatabase) -> LitoolsResult<AppSettings> {
    let connection = database.connection();
    let repository = SettingsRepository::new(&connection);
    let settings = match repository.get_json(APP_SETTINGS_KEY)? {
        Some(value_json) => serde_json::from_str::<AppSettings>(&value_json)
            .unwrap_or_else(|_| AppSettings::default())
            .normalized(),
        None => AppSettings::default(),
    };
    repository.set_json(APP_SETTINGS_KEY, &serde_json::to_string(&settings)?)?;
    Ok(settings)
}

fn persist_settings(database: &IndexDatabase, settings: &AppSettings) -> LitoolsResult<()> {
    let connection = database.connection();
    SettingsRepository::new(&connection)
        .set_json(APP_SETTINGS_KEY, &serde_json::to_string(settings)?)?;
    Ok(())
}

fn builtin_effect_for_result(result_id: &str) -> LitoolsResult<CommandEffect> {
    match result_id {
        "open-logs-directory" => Ok(CommandEffect::OpenLogsDirectory),
        "open-data-directory" => Ok(CommandEffect::OpenDataDirectory),
        "reload-index" => Ok(CommandEffect::ReloadIndex),
        "quit-app" => Ok(CommandEffect::QuitApp),
        "toggle-theme" => Ok(CommandEffect::ToggleTheme),
        _ => Err(LitoolsError::CommandNotFound(result_id.to_string())),
    }
}

fn message_for_effect(effect: &CommandEffect) -> &'static str {
    match effect {
        CommandEffect::None => "未执行任何操作",
        CommandEffect::OpenLogsDirectory => "正在打开日志目录",
        CommandEffect::OpenDataDirectory => "正在打开数据目录",
        CommandEffect::OpenPluginView { .. } => "正在打开插件",
        CommandEffect::ReloadIndex => "正在重载索引",
        CommandEffect::QuitApp => "正在退出应用",
        CommandEffect::ToggleTheme => "正在切换主题",
    }
}

#[cfg(test)]
mod tests {
    use litools_settings::{PaletteSettings, SearchSettings, WindowSettings};

    use super::*;

    #[test]
    fn bootstrap_writes_default_settings() {
        let app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        assert_eq!(app.settings(), &AppSettings::default());
    }

    #[test]
    fn launcher_panel_respects_section_settings() {
        let mut app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");
        let mut settings = app.settings().clone();
        settings.palette.show_recent = false;
        settings.palette.show_pinned = false;
        app.update_settings(settings).expect("update settings");

        let panel = app.launcher_panel("").expect("launcher panel");

        assert!(panel.sections.is_empty());
    }

    #[test]
    fn pin_result_adds_pinned_section_item() {
        let app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        app.pin_result("reload-index").expect("pin command");
        let panel = app.launcher_panel("").expect("launcher panel");

        let pinned = panel
            .sections
            .iter()
            .find(|section| section.id == "pinned")
            .expect("pinned section");
        assert_eq!(pinned.items[0].result.id, "reload-index");
        assert!(pinned.items[0].is_pinned);

        app.unpin_result("reload-index").expect("unpin command");
        let panel = app.launcher_panel("").expect("launcher panel");
        assert!(panel.sections.iter().all(|section| section.id != "pinned"));
    }

    #[test]
    fn reorder_pinned_results_updates_launcher_order() {
        let app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        app.pin_result("reload-index").expect("pin reload");
        app.pin_result("quit-app").expect("pin quit");
        app.reorder_pinned_results(vec![
            "reload-index".to_string(),
            "quit-app".to_string(),
        ])
        .expect("reorder pinned results");

        let panel = app.launcher_panel("").expect("launcher panel");
        let pinned = panel
            .sections
            .iter()
            .find(|section| section.id == "pinned")
            .expect("pinned section");
        let ids = pinned
            .items
            .iter()
            .map(|item| item.result.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["reload-index", "quit-app"]);
    }

    #[test]
    fn launcher_panel_keeps_recent_above_pinned_without_deduplication() {
        let mut app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        app.execute_result("reload-index", "execute")
            .expect("record recent usage");
        app.pin_result("reload-index").expect("pin settings");
        let panel = app.launcher_panel("").expect("launcher panel");

        assert_eq!(panel.sections[0].id, "recent");
        assert_eq!(panel.sections[1].id, "pinned");
        assert_eq!(panel.sections[0].items[0].result.id, "reload-index");
        assert_eq!(panel.sections[1].items[0].result.id, "reload-index");
        assert!(panel.sections[0].items[0].is_pinned);
        assert!(panel.sections[1].items[0].is_pinned);
    }

    #[test]
    fn update_settings_persists_normalized_settings() {
        let mut app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");
        let settings = AppSettings {
            theme: "invalid".to_string(),
            palette: PaletteSettings {
                global_hotkey: "".to_string(),
                show_recent: false,
                show_pinned: false,
            },
            search: SearchSettings {
                enabled_providers: vec![],
            },
            window: WindowSettings {
                hide_on_blur: false,
                close_to_tray: false,
                center_on_show: false,
            },
        };

        let updated = app.update_settings(settings).expect("update settings");

        assert_eq!(updated.theme, "system");
        assert_eq!(
            updated.search.enabled_providers,
            ["apps", "commands", "plugins"]
        );
        assert_eq!(app.settings(), &updated);
    }

    #[test]
    fn toggle_theme_persists_theme_change() {
        let mut app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        app.execute_result("toggle-theme", "execute")
            .expect("toggle theme");

        assert_eq!(app.settings().theme, "dark");
        assert_eq!(app.usage_event_count().expect("usage count"), 1);
    }
}
