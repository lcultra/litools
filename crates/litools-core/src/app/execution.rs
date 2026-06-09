use chrono::Utc;
use litools_index::repository::{AppRepository, UsageRepository};
use litools_plugin::{PluginCommandMode, plugin_target_id};
use litools_system::{NativeSystemAdapter, SystemAdapter};
use uuid::Uuid;

use litools_config::search::ACTION_OPEN;

use crate::{
    app::LitoolsApp,
    command::{CommandEffect, CommandExecution, builtin_effect_for_result, message_for_effect},
    error::{LitoolsError, LitoolsResult},
};

use super::plugins::plugin_route;

impl LitoolsApp {
    pub fn execute_result(
        &mut self,
        result_id: impl Into<String>,
        action_id: impl Into<String>,
    ) -> LitoolsResult<CommandExecution> {
        let result_id = result_id.into();
        let action_id = action_id.into();

        if let Some(app_id) =
            crate::app_provider::app_id_from_result_id(&result_id)
        {
            return self.execute_app_result(&result_id, app_id, &action_id);
        }

        if let Some((plugin_id, command_id)) =
            litools_plugin::plugin_command_from_result_id(&result_id)
        {
            return self.execute_plugin_command_result(
                &result_id,
                plugin_id,
                command_id,
                &action_id,
            );
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

    fn execute_app_result(
        &self,
        result_id: &str,
        app_id: &str,
        action_id: &str,
    ) -> LitoolsResult<CommandExecution> {
        if action_id != ACTION_OPEN {
            return Err(LitoolsError::CommandNotFound(format!(
                "{result_id}:{action_id}"
            )));
        }

        let connection = self.context.database.connection();
        let apps = AppRepository::new(&connection);
        let app = apps
            .find_app(app_id)?
            .ok_or_else(|| LitoolsError::AppNotFound(result_id.to_string()))?;

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

    fn execute_plugin_command_result(
        &self,
        result_id: &str,
        plugin_id: &str,
        command_id: &str,
        action_id: &str,
    ) -> LitoolsResult<CommandExecution> {
        if action_id != ACTION_OPEN {
            return Err(LitoolsError::CommandNotFound(format!(
                "{result_id}:{action_id}"
            )));
        }

        let plugin = self
            .context
            .plugins
            .find_plugin(plugin_id)
            .ok_or_else(|| LitoolsError::PluginNotFound(result_id.to_string()))?;
        if !plugin.enabled {
            return Err(LitoolsError::PluginDisabled(result_id.to_string()));
        }
        let command = plugin
            .manifest
            .commands
            .iter()
            .find(|command| command.id == command_id)
            .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;
        if command.mode != PluginCommandMode::View {
            return Err(LitoolsError::InvalidAction(format!(
                "plugin command is not a view: {result_id}"
            )));
        }

        let route = plugin_route(plugin_id, command_id);
        let connection = self.context.database.connection();
        UsageRepository::new(&connection).record_selection(
            &Uuid::new_v4().to_string(),
            litools_plugin::PLUGIN_TARGET_TYPE,
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
}
