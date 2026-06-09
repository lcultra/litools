use chrono::{DateTime, Utc};
use litools_index::repository::{
    AppRecord, AppRepository, AppUpsert, CommandRepository, IndexMetadataRepository,
    UsageEventRecord, UsageRepository,
};
use litools_system::{NativeSystemAdapter, SystemAdapter};

use crate::{
    app::{LitoolsApp, ReloadIndexSummary},
    command::BUILTIN_COMMANDS,
    error::LitoolsResult,
};

use super::{APPS_INDEX_STATUS_KEY, RELOAD_INDEX_TRIGGER_DIRECT};

impl LitoolsApp {
    pub fn reload_index(&mut self) -> LitoolsResult<ReloadIndexSummary> {
        self.reload_index_with_trigger(RELOAD_INDEX_TRIGGER_DIRECT)
    }

    pub fn reload_index_with_trigger(
        &mut self,
        trigger: &str,
    ) -> LitoolsResult<ReloadIndexSummary> {
        let started_at = Utc::now();
        let discovered_apps = NativeSystemAdapter.discover_apps();
        let apps_discovered = discovered_apps.len();

        self.context.plugins =
            super::plugins::sync_and_load_plugins(&self.context.database, &self.paths)?;
        self.plugin_command_provider.invalidate_cache();

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

        let apps_removed =
            apps.delete_apps_not_seen_at(std::env::consts::OS, &last_seen_at)?;
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
