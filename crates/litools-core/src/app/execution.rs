use litools_config::search::ResultId;

use crate::{
    app::LitoolsApp,
    command::{CommandEffect, CommandExecution, builtin_effect_for_result, message_for_effect},
    error::{LitoolsError, LitoolsResult},
    execution_service,
};

impl LitoolsApp {
    /// 执行一个搜索结果的操作。根据 ResultId 分派到 App/Plugin/Builtin 三条路径。
    ///
    /// `provider` 对应 `SearchResult.provider`，由前端传入，用于 ExecutorRegistry 路由。
    pub fn execute_result(
        &mut self,
        result_id: impl Into<String>,
        action_id: impl Into<String>,
        provider: impl Into<String>,
    ) -> LitoolsResult<CommandExecution> {
        let result_id = result_id.into();
        let action_id = action_id.into();
        let provider = provider.into();

        let parsed = ResultId::parse(&result_id)
            .ok_or_else(|| LitoolsError::CommandNotFound(result_id.clone()))?;

        match parsed {
            ResultId::App(_) => {
                // 委托给 ExecutorRegistry，按前端传入的 provider 路由
                self.context.executor_registry.execute(
                    &provider,
                    &result_id,
                    &action_id,
                    &self.context,
                )
            }
            ResultId::Plugin {
                plugin_id,
                command_id,
            } => execution_service::execute_plugin(
                &self.context.plugins,
                &self.context.database,
                &result_id,
                &plugin_id,
                &command_id,
                &action_id,
            ),
            ResultId::Builtin(id) => {
                let effect = builtin_effect_for_result(&id)?;

                match effect {
                    CommandEffect::ReloadIndex => {
                        let _ = self.reload_index()?;
                    }
                    CommandEffect::ToggleTheme => self.toggle_theme()?,
                    _ => {}
                }

                execution_service::record_builtin_usage(&self.context.database, &id)?;

                Ok(CommandExecution {
                    message: message_for_effect(&effect).to_string(),
                    result_id,
                    action_id,
                    effect,
                })
            }
        }
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
