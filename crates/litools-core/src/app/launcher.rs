use litools_index::repository::{AppRepository, PinnedRepository, PluginCommandRepository};
use litools_plugin::PLUGIN_TARGET_TYPE;
use litools_search::SearchResult;

use crate::{
    app::LitoolsApp,
    app_provider::search_result_for_app,
    command::{find_builtin_command, search_result_for_builtin_command},
    error::LitoolsResult,
    launcher::{LauncherItem, LauncherSection},
    plugin_provider::search_result_for_plugin_command_record,
};

use super::DEFAULT_LAUNCHER_RESULT_LIMIT;

impl LitoolsApp {
    pub fn launcher_panel(
        &self,
        query: impl Into<String>,
    ) -> LitoolsResult<crate::launcher::LauncherPanelResponse> {
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

            return Ok(crate::launcher::LauncherPanelResponse {
                sections: section_if_not_empty("best", "最佳搜索结果", items)
                    .into_iter()
                    .collect(),
            });
        }

        let settings = self.context.settings.get();
        let mut sections = Vec::new();

        if settings.palette.show_recent {
            let recent_items = self.recent_launcher_items(DEFAULT_LAUNCHER_RESULT_LIMIT)?;

            if let Some(section) = section_if_not_empty("recent", "最近使用", recent_items) {
                sections.push(section);
            }
        }

        if settings.palette.show_pinned {
            let pinned_items = self.pinned_launcher_items(DEFAULT_LAUNCHER_RESULT_LIMIT)?;

            if let Some(section) = section_if_not_empty("pinned", "已固定", pinned_items) {
                sections.push(section);
            }
        }

        Ok(crate::launcher::LauncherPanelResponse { sections })
    }

    pub(crate) fn pinned_launcher_items(&self, limit: usize) -> LitoolsResult<Vec<LauncherItem>> {
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

    pub(crate) fn recent_launcher_items(&self, limit: usize) -> LitoolsResult<Vec<LauncherItem>> {
        use litools_index::repository::UsageRepository;

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
}

pub(crate) fn section_if_not_empty(
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

pub(crate) fn result_for_target(
    apps: &AppRepository<'_>,
    plugin_commands: &PluginCommandRepository<'_>,
    target_type: &str,
    target_id: &str,
) -> LitoolsResult<Option<SearchResult>> {
    use litools_plugin::plugin_command_from_target_id;

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
