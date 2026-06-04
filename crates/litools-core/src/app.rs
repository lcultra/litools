use std::{path::Path, sync::Arc};

use chrono::Utc;
use litools_index::{
    IndexDatabase,
    repository::{CommandRepository, SettingsRepository, UsageEventRecord, UsageRepository},
};
use litools_search::{SearchEngine, SearchQuery, SearchResult};
use litools_settings::{AppSettings, storage::SettingsStore};
use litools_telemetry::init_logging;
use uuid::Uuid;

const APP_SETTINGS_KEY: &str = "app_settings";

use crate::{
    command::{BUILTIN_COMMANDS, BuiltinCommandEffect, BuiltinCommandProvider, CommandExecution},
    context::AppContext,
    error::{LitoolsError, LitoolsResult},
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

        Ok(Self {
            context: AppContext::new(
                database,
                default_search_engine(),
                SettingsStore::new(settings),
            ),
        })
    }

    pub fn bootstrap_in_memory() -> LitoolsResult<Self> {
        init_logging();

        let database = IndexDatabase::in_memory()?;
        let settings = load_settings(&database)?;

        Ok(Self {
            context: AppContext::new(
                database,
                default_search_engine(),
                SettingsStore::new(settings),
            ),
        })
    }

    pub fn search(&self, text: impl Into<String>) -> Vec<SearchResult> {
        let settings = self.context.settings.get();
        self.context.search.search_with_providers(
            SearchQuery::with_limit(text, settings.palette.result_limit),
            settings.search.enabled_providers.iter().map(String::as_str),
        )
    }

    pub fn settings(&self) -> &AppSettings {
        self.context.settings.get()
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
        let effect = builtin_effect_for_result(&result_id)?;

        match effect {
            BuiltinCommandEffect::ReloadIndex => self.reload_index()?,
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

    pub fn reload_index(&self) -> LitoolsResult<()> {
        let connection = self.context.database.connection();
        let commands = CommandRepository::new(&connection);

        for command in BUILTIN_COMMANDS {
            commands.upsert_command(
                command.id,
                "builtin",
                command.title,
                Some(command.subtitle),
                "execute",
            )?;
        }

        Ok(())
    }

    pub fn recent_usage_events(&self, limit: usize) -> LitoolsResult<Vec<UsageEventRecord>> {
        let connection = self.context.database.connection();
        Ok(UsageRepository::new(&connection).recent_events(limit)?)
    }

    pub fn command_count(&self) -> LitoolsResult<usize> {
        let connection = self.context.database.connection();
        Ok(CommandRepository::new(&connection).count_commands()?)
    }

    pub fn usage_event_count(&self) -> LitoolsResult<usize> {
        let connection = self.context.database.connection();
        Ok(UsageRepository::new(&connection).count_events()?)
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

fn default_search_engine() -> SearchEngine {
    let mut search = SearchEngine::new();
    search.register_provider(Arc::new(BuiltinCommandProvider));
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
        "reload-index" => Ok(BuiltinCommandEffect::ReloadIndex),
        "open-logs" => Ok(BuiltinCommandEffect::OpenLogs),
        "quit-app" => Ok(BuiltinCommandEffect::QuitApp),
        "toggle-theme" => Ok(BuiltinCommandEffect::ToggleTheme),
        _ => Err(LitoolsError::CommandNotFound(result_id.to_string())),
    }
}

fn message_for_effect(effect: &BuiltinCommandEffect) -> &'static str {
    match effect {
        BuiltinCommandEffect::None => "未执行任何操作",
        BuiltinCommandEffect::OpenSettings => "正在打开设置",
        BuiltinCommandEffect::ReloadIndex => "正在重载索引",
        BuiltinCommandEffect::OpenLogs => "正在打开诊断",
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
    fn update_settings_persists_normalized_settings() {
        let mut app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");
        let settings = AppSettings {
            theme: "invalid".to_string(),
            palette: PaletteSettings {
                global_hotkey: "".to_string(),
                result_limit: 100,
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
        assert_eq!(updated.palette.result_limit, 50);
        assert_eq!(updated.search.enabled_providers, ["commands"]);
        assert_eq!(app.settings(), &updated);
    }

    #[test]
    fn search_uses_settings_result_limit() {
        let mut app = LitoolsApp::bootstrap_in_memory().expect("bootstrap app");
        let mut settings = app.settings().clone();
        settings.palette.result_limit = 1;
        app.update_settings(settings).expect("update settings");

        let results = app.search("");

        assert_eq!(results.len(), 1);
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
