use chrono::Utc;
use litools_config::search::ResultId;
use litools_index::repository::{AppRepository, PinnedRepository, PluginCommandRepository};

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
        let parsed =
            ResultId::parse(result_id)
                .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;

        // 验证目标实体真实存在
        match &parsed {
            ResultId::App(app_id) => {
                let connection = self.context.database.connection();
                AppRepository::new(&connection)
                    .find_app(app_id)?
                    .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;
            }
            ResultId::Builtin(id) => {
                find_builtin_command(id)
                    .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;
            }
            ResultId::Plugin {
                plugin_id,
                command_id,
            } => {
                let connection = self.context.database.connection();
                PluginCommandRepository::new(&connection)
                    .find_plugin_command(plugin_id, command_id)?
                    .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;
            }
        }

        Ok(parsed.to_target())
    }

    /// 将 result_id 解析为 `(target_type, target_id)`，不做存在性验证。
    ///
    /// 用于 [`launcher_panel`] 中的 `is_pinned` 快速检查——搜索结果已保证 ID 有效，
    /// 无需额外查询数据库。
    pub(crate) fn target_from_result_id(&self, result_id: &str) -> Option<(&'static str, String)> {
        ResultId::parse(result_id).map(|r| r.to_target())
    }
}
