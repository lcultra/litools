use chrono::Utc;
use litools_config::search::ResultId;
use litools_index::{
    IndexDatabase,
    repository::{AppRepository, PinnedRepository, PluginCommandRepository},
};

use crate::{
    command::find_builtin_command,
    error::{LitoolsError, LitoolsResult},
};

/// 验证 result_id 对应的实体存在，返回 `(target_type, target_id)` 用于 pinned_items 表。
pub fn validate_target(database: &IndexDatabase, result_id: &str) -> LitoolsResult<(&'static str, String)> {
    let parsed =
        ResultId::parse(result_id)
            .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;

    match &parsed {
        ResultId::App(app_id) => {
            database
                .read(|conn| AppRepository::new(&conn).find_app(app_id))?
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
            database
                .read(|conn| {
                    PluginCommandRepository::new(&conn).find_plugin_command(plugin_id, command_id)
                })?
                .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;
        }
    }

    Ok(parsed.to_target())
}

/// 将 result_id 解析为 `(target_type, target_id)`，不做存在性验证。
pub fn parse_target(result_id: &str) -> Option<(&'static str, String)> {
    ResultId::parse(result_id).map(|r| r.to_target())
}

/// 固定一个目标。
pub fn pin(database: &IndexDatabase, target_type: &str, target_id: &str) -> LitoolsResult<()> {
    database.read(|conn| {
        PinnedRepository::new(&conn).pin(target_type, target_id, &Utc::now().to_rfc3339())
    })?;
    Ok(())
}

/// 取消固定一个目标。
pub fn unpin(database: &IndexDatabase, target_type: &str, target_id: &str) -> LitoolsResult<()> {
    database.read(|conn| PinnedRepository::new(&conn).unpin(target_type, target_id))?;
    Ok(())
}

/// 检查一个目标是否已固定。
pub fn is_pinned(database: &IndexDatabase, target_type: &str, target_id: &str) -> LitoolsResult<bool> {
    Ok(database.read(|conn| PinnedRepository::new(&conn).is_pinned(target_type, target_id))?)
}

/// 重新排序已固定项目。调用方负责验证目标存在且已固定。
pub fn reorder(database: &IndexDatabase, ordered_targets: &[(String, String)]) -> LitoolsResult<()> {
    database.read(|conn| {
        PinnedRepository::new(&conn).reorder(ordered_targets)
    })?;
    Ok(())
}
