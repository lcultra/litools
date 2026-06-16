use chrono::{DateTime, Utc};
use litools_config::app::APPS_INDEX_STATUS_KEY;
use litools_index::{
    IndexDatabase,
    repository::{
        AppRecord, AppRepository, CommandRepository, IndexMetadataRepository, UsageEventRecord,
        UsageRepository,
    },
};

use crate::{ReloadIndexSummary, error::LitoolsResult};

/// 最近的 usage 事件。
pub fn recent_usage_events(
    database: &IndexDatabase,
    limit: usize,
) -> LitoolsResult<Vec<UsageEventRecord>> {
    Ok(database
        .read(|conn| UsageRepository::new(&conn).recent_events(limit))?)
}

/// 按 ID 查找应用。
pub fn find_app(database: &IndexDatabase, id: &str) -> LitoolsResult<Option<AppRecord>> {
    Ok(database.read(|conn| AppRepository::new(&conn).find_app(id))?)
}

/// 命令总数。
pub fn command_count(database: &IndexDatabase) -> LitoolsResult<usize> {
    Ok(database.read(|conn| CommandRepository::new(&conn).count_commands())?)
}

/// 应用总数。
pub fn app_count(database: &IndexDatabase) -> LitoolsResult<usize> {
    Ok(database.read(|conn| AppRepository::new(&conn).count_apps())?)
}

/// 最近一次索引状态摘要。
pub fn index_status(database: &IndexDatabase) -> LitoolsResult<Option<ReloadIndexSummary>> {
    let Some(metadata) = database
        .read(|conn| IndexMetadataRepository::new(&conn).get(APPS_INDEX_STATUS_KEY))?
    else {
        return Ok(None);
    };
    Ok(Some(serde_json::from_str(&metadata.value_json)?))
}

/// 使用事件总数。
pub fn usage_event_count(database: &IndexDatabase) -> LitoolsResult<usize> {
    Ok(database.read(|conn| UsageRepository::new(&conn).count_events())?)
}

pub(crate) fn build_summary(
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
