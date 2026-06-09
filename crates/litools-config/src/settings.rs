//! 设置默认值常量。

/// 默认全局快捷键，可在设置中修改。
pub const DEFAULT_GLOBAL_HOTKEY: &str = "CommandOrControl+Space";
/// 默认启用的搜索提供商列表。新用户/重置设置后生效。
pub const DEFAULT_ENABLED_PROVIDERS: &[&str] = &["apps", "commands", "plugins"];
/// 系统支持的所有搜索提供商（白名单）。用户设置中不在白名单的值会被过滤掉。
pub const SUPPORTED_SEARCH_PROVIDERS: &[&str] = &["apps", "commands", "plugins"];
