//! 搜索相关常量：provider 标识、结果 ID 前缀、默认限制。

// ── 搜索规模 ──

/// 启动器搜索结果最大条数，未来可由用户配置。
pub const DEFAULT_LAUNCHER_RESULT_LIMIT: usize = 20;

// ── Provider 标识 ──
//
// 每个 SearchProvider 通过 `id()` 返回一个唯一标识，搜索引擎用它匹配
// 用户设置中启用的 provider 列表。

/// 系统应用搜索 provider。
pub const APP_PROVIDER_ID: &str = "apps";
/// 插件命令搜索 provider。
pub const PLUGIN_PROVIDER_ID: &str = "plugins";
/// 内置命令搜索 provider。
pub const COMMAND_PROVIDER_ID: &str = "commands";

// ── 结果 ID 前缀 ──
//
// 搜索结果通过 `{prefix}{实体标识}` 来区分类型，方便执行时路由。

/// 应用结果 ID 前缀，格式：`app:{bundle_id}` 或 `app:path:/...`
pub const APP_RESULT_PREFIX: &str = "app:";
/// 插件结果 ID 前缀，格式：`plugin:{plugin_id}:{command_id}`
pub const PLUGIN_RESULT_PREFIX: &str = "plugin:";

// ── 操作标识 ──

/// 打开操作：启动应用、打开插件视图。
pub const ACTION_OPEN: &str = "open";
/// 执行操作：触发内置命令（如重载索引、切换主题）。
pub const ACTION_EXECUTE: &str = "execute";
