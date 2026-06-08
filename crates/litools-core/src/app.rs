use std::{path::Path, sync::Arc};

use chrono::{DateTime, Utc};
use litools_index::{
    IndexDatabase,
    repository::{
        AppRecord, AppRepository, AppUpsert, CommandRepository, IndexMetadataRepository,
        PinnedRepository, SettingsRepository, UsageEventRecord, UsageRepository,
    },
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
        BUILTIN_COMMANDS, BuiltinCommandEffect, BuiltinCommandProvider, CommandExecution,
        find_builtin_command, search_result_for_builtin_command,
    },
    context::AppContext,
    error::{LitoolsError, LitoolsResult},
    launcher::{LauncherItem, LauncherPanelResponse, LauncherSection},
};

pub struct LitoolsApp {
    context: AppContext,
}

impl LitoolsApp {
    pub fn bootstrap(data_dir: impl AsRef<Path>) -> LitoolsResult<Self> {
        init_logging();

        std::fs::create_dir_all(data_dir.as_ref())?;
        let database = IndexDatabase::open(data_dir.as_ref().join("database.sqlite"))?;
        let settings = load_settings(&database)?;
        let search = default_search_engine(database.clone());

        Ok(Self {
            context: AppContext::new(database, search, SettingsStore::new(settings)),
        })
    }

    pub fn bootstrap_in_memory() -> LitoolsResult<Self> {
        init_logging();

        let database = IndexDatabase::in_memory()?;
        let settings = load_settings(&database)?;
        let search = default_search_engine(database.clone());

        Ok(Self {
            context: AppContext::new(database, search, SettingsStore::new(settings)),
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

        let effect = builtin_effect_for_result(&result_id)?;

        match effect {
            BuiltinCommandEffect::ReloadIndex => {
                let _ = self.reload_index()?;
            }
            BuiltinCommandEffect::ToggleTheme => self.toggle_theme()?,
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

    pub fn reload_index(&self) -> LitoolsResult<ReloadIndexSummary> {
        self.reload_index_with_trigger(RELOAD_INDEX_TRIGGER_DIRECT)
    }

    pub fn reload_index_with_trigger(&self, trigger: &str) -> LitoolsResult<ReloadIndexSummary> {
        let started_at = Utc::now();
        let discovered_apps = NativeSystemAdapter.discover_apps();
        let apps_discovered = discovered_apps.len();
        let connection = self.context.database.connection();
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
        let mut items = Vec::new();

        for record in pinned {
            if let Some(result) = result_for_target(&apps, &record.target_type, &record.target_id)?
            {
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
        let pinned = PinnedRepository::new(&connection);
        let mut items = Vec::new();

        for record in usage {
            let Some(result) = result_for_target(&apps, &record.target_type, &record.target_id)?
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
            _ => return Err(LitoolsError::CommandNotFound(result_id.to_string())),
        }

        Ok((target_type, target_id))
    }

    fn target_from_result_id(&self, result_id: &str) -> Option<(&'static str, String)> {
        if let Some(app_id) = app_id_from_result_id(result_id) {
            return Some(("app", app_id.to_string()));
        }

        find_builtin_command(result_id).map(|command| ("command", command.id.to_string()))
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
            effect: BuiltinCommandEffect::None,
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
    target_type: &str,
    target_id: &str,
) -> LitoolsResult<Option<SearchResult>> {
    match target_type {
        "app" => Ok(apps
            .find_app(target_id)?
            .map(|app| search_result_for_app(app, ""))),
        "command" => Ok(find_builtin_command(target_id)
            .map(|command| search_result_for_builtin_command(command, ""))),
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
    search.register_provider(Arc::new(AppSearchProvider::new(database)));
    search
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

fn builtin_effect_for_result(result_id: &str) -> LitoolsResult<BuiltinCommandEffect> {
    match result_id {
        "open-settings" => Ok(BuiltinCommandEffect::OpenSettings),
        "open-diagnostics" => Ok(BuiltinCommandEffect::OpenDiagnostics),
        "open-plugins" => Ok(BuiltinCommandEffect::OpenPlugins),
        "open-logs-directory" => Ok(BuiltinCommandEffect::OpenLogsDirectory),
        "open-data-directory" => Ok(BuiltinCommandEffect::OpenDataDirectory),
        "reload-index" => Ok(BuiltinCommandEffect::ReloadIndex),
        "quit-app" => Ok(BuiltinCommandEffect::QuitApp),
        "toggle-theme" => Ok(BuiltinCommandEffect::ToggleTheme),
        _ => Err(LitoolsError::CommandNotFound(result_id.to_string())),
    }
}

fn message_for_effect(effect: &BuiltinCommandEffect) -> &'static str {
    match effect {
        BuiltinCommandEffect::None => "未执行任何操作",
        BuiltinCommandEffect::OpenSettings => "正在打开设置",
        BuiltinCommandEffect::OpenDiagnostics => "正在打开诊断",
        BuiltinCommandEffect::OpenPlugins => "正在打开插件管理",
        BuiltinCommandEffect::OpenLogsDirectory => "正在打开日志目录",
        BuiltinCommandEffect::OpenDataDirectory => "正在打开数据目录",
        BuiltinCommandEffect::ReloadIndex => "正在重载索引",
        BuiltinCommandEffect::QuitApp => "正在退出应用",
        BuiltinCommandEffect::ToggleTheme => "正在切换主题",
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

        app.pin_result("open-settings").expect("pin command");
        let panel = app.launcher_panel("").expect("launcher panel");

        let pinned = panel
            .sections
            .iter()
            .find(|section| section.id == "pinned")
            .expect("pinned section");
        assert_eq!(pinned.items[0].result.id, "open-settings");
        assert!(pinned.items[0].is_pinned);

        app.unpin_result("open-settings").expect("unpin command");
        let panel = app.launcher_panel("").expect("launcher panel");
        assert!(panel.sections.iter().all(|section| section.id != "pinned"));
    }

    #[test]
    fn reorder_pinned_results_updates_launcher_order() {
        let app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        app.pin_result("open-settings").expect("pin settings");
        app.pin_result("reload-index").expect("pin reload");
        app.reorder_pinned_results(vec![
            "open-settings".to_string(),
            "reload-index".to_string(),
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

        assert_eq!(ids, vec!["open-settings", "reload-index"]);
    }

    #[test]
    fn launcher_panel_keeps_recent_above_pinned_without_deduplication() {
        let mut app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");

        app.execute_result("open-settings", "execute")
            .expect("record recent usage");
        app.pin_result("open-settings").expect("pin settings");
        let panel = app.launcher_panel("").expect("launcher panel");

        assert_eq!(panel.sections[0].id, "recent");
        assert_eq!(panel.sections[1].id, "pinned");
        assert_eq!(panel.sections[0].items[0].result.id, "open-settings");
        assert_eq!(panel.sections[1].items[0].result.id, "open-settings");
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
        assert_eq!(updated.search.enabled_providers, ["apps", "commands"]);
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
