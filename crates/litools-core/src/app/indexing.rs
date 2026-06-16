use std::sync::Arc;

use chrono::Utc;
use litools_index::repository::{AppRepository, CommandRepository, IndexMetadataRepository};
use litools_system::{NativeSystemAdapter, SystemAdapter};

use crate::{
    app::{LitoolsApp, ReloadIndexSummary},
    command::BUILTIN_COMMANDS,
    error::LitoolsResult,
    indexing_service,
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

        if trigger != "startup" {
            let manager = Arc::new(super::plugins::sync_and_load_plugins(
                &self.context.database,
                &self.paths,
            )?);
            self.context.plugins = manager.clone();
            self.plugin_command_provider.set_plugin_manager(manager);
        }

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
            apps.upsert_app(litools_index::repository::AppUpsert {
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
        let summary = indexing_service::build_summary(
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

        log::info!(
            "索引重建完成: {} 个应用, {} 个命令, 耗时 {}ms",
            summary.apps_upserted,
            summary.commands_upserted,
            summary.duration_ms,
        );
        Ok(summary)
    }

    pub fn recent_usage_events(
        &self,
        limit: usize,
    ) -> LitoolsResult<Vec<litools_index::repository::UsageEventRecord>> {
        indexing_service::recent_usage_events(&self.context.database, limit)
    }

    pub fn find_app(
        &self,
        id: &str,
    ) -> LitoolsResult<Option<litools_index::repository::AppRecord>> {
        indexing_service::find_app(&self.context.database, id)
    }

    pub fn command_count(&self) -> LitoolsResult<usize> {
        indexing_service::command_count(&self.context.database)
    }

    pub fn app_count(&self) -> LitoolsResult<usize> {
        indexing_service::app_count(&self.context.database)
    }

    pub fn index_status(&self) -> LitoolsResult<Option<ReloadIndexSummary>> {
        indexing_service::index_status(&self.context.database)
    }

    pub fn usage_event_count(&self) -> LitoolsResult<usize> {
        indexing_service::usage_event_count(&self.context.database)
    }
}
