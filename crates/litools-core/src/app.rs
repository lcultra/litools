use std::{path::Path, sync::Arc};

use chrono::Utc;
use litools_index::{
    IndexDatabase,
    repository::{CommandRepository, UsageEventRecord, UsageRepository},
};
use litools_search::{SearchEngine, SearchQuery, SearchResult};
use litools_telemetry::init_logging;
use uuid::Uuid;

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

        Ok(Self {
            context: AppContext::new(database, default_search_engine()),
        })
    }

    pub fn bootstrap_in_memory() -> LitoolsResult<Self> {
        init_logging();

        Ok(Self {
            context: AppContext::new(IndexDatabase::in_memory()?, default_search_engine()),
        })
    }

    pub fn search(&self, text: impl Into<String>) -> Vec<SearchResult> {
        self.context.search.search(SearchQuery::new(text))
    }

    pub fn execute_result(
        &self,
        result_id: impl Into<String>,
        action_id: impl Into<String>,
    ) -> LitoolsResult<CommandExecution> {
        let result_id = result_id.into();
        let action_id = action_id.into();
        let effect = builtin_effect_for_result(&result_id)?;

        if matches!(effect, BuiltinCommandEffect::ReloadIndex) {
            self.reload_index()?;
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

    pub fn context(&self) -> &AppContext {
        &self.context
    }
}

fn default_search_engine() -> SearchEngine {
    let mut search = SearchEngine::new();
    search.register_provider(Arc::new(BuiltinCommandProvider));
    search
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
        BuiltinCommandEffect::None => "No action executed",
        BuiltinCommandEffect::OpenSettings => "Opening settings",
        BuiltinCommandEffect::ReloadIndex => "Reloading index",
        BuiltinCommandEffect::OpenLogs => "Opening logs",
        BuiltinCommandEffect::QuitApp => "Quitting app",
        BuiltinCommandEffect::ToggleTheme => "Toggling theme",
    }
}
