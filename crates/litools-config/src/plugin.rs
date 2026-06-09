//! 插件相关常量。

/// 插件清单文件名，位于插件根目录下。discovery 模块扫描目录时按此文件名查找。
pub const PLUGIN_MANIFEST_FILE_NAME: &str = "plugin.json";
/// 插件目标的类型标识，用于 pinned_items / usage_events 表中的 target_type 字段。
pub const PLUGIN_TARGET_TYPE: &str = "plugin_command";
/// 插件存储 key 最大长度（字节），防止恶意插件写入超长 key。
pub const MAX_STORAGE_KEY_LEN: usize = 256;
/// 插件存储 value 最大长度（字节）= 256KB，防止恶意插件撑爆数据库。
pub const MAX_STORAGE_VALUE_LEN: usize = 256 * 1024;
