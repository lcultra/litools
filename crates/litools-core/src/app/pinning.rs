use chrono::Utc;
use litools_index::repository::{AppRepository, PinnedRepository, PluginCommandRepository};
use litools_plugin::PLUGIN_TARGET_TYPE;

use crate::{
    app::LitoolsApp,
    command::find_builtin_command,
    error::{LitoolsError, LitoolsResult},
};

impl LitoolsApp {
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

    pub(crate) fn is_target_pinned(
        &self,
        target_type: &str,
        target_id: &str,
    ) -> LitoolsResult<bool> {
        let connection = self.context.database.connection();
        Ok(PinnedRepository::new(&connection).is_pinned(target_type, target_id)?)
    }

    pub(crate) fn validated_target_from_result_id(
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
                let Some((plugin_id, command_id)) =
                    litools_plugin::plugin_command_from_target_id(&target_id)
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

    pub(crate) fn target_from_result_id(&self, result_id: &str) -> Option<(&'static str, String)> {
        use crate::app_provider::app_id_from_result_id;
        use litools_plugin::{plugin_command_from_result_id, plugin_target_id};

        if let Some(app_id) = app_id_from_result_id(result_id) {
            return Some(("app", app_id.to_string()));
        }

        if let Some((plugin_id, command_id)) = plugin_command_from_result_id(result_id) {
            return Some((PLUGIN_TARGET_TYPE, plugin_target_id(plugin_id, command_id)));
        }

        find_builtin_command(result_id).map(|command| ("command", command.id.to_string()))
    }
}
