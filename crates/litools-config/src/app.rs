//! 应用级常量：数据库 key、触发器标识。

/// 应用设置持久化的 key，对应 settings 表。
pub const APP_SETTINGS_KEY: &str = "app_settings";
/// 索引状态持久化的 key，对应 index_metadata 表。存储最近一次索引重建的摘要。
pub const APPS_INDEX_STATUS_KEY: &str = "apps_last_refresh_status";
/// 用户手动触发索引重载的 trigger 标识，区别于 startup 和 appWatcher。
pub const RELOAD_INDEX_TRIGGER_DIRECT: &str = "direct";
