//! 图标相关常量：缓存配置、转换尺寸。

/// 内存图标缓存最大条目数（LRU），未来可由用户配置。
pub const DEFAULT_ICON_CACHE_CAPACITY: usize = 128;
/// 图标转换目标像素尺寸。.icns 文件取最接近此尺寸的图标转 PNG。
pub const DEFAULT_ICON_TARGET_SIZE: u32 = 128;
/// 磁盘图标缓存最大文件数，超出后按修改时间清理，未来可由用户配置。
pub const DEFAULT_ICON_CACHE_MAX_FILES: usize = 1024;
/// 磁盘图标缓存最大字节数（128MB），超出后按修改时间清理，未来可由用户配置。
pub const DEFAULT_ICON_CACHE_MAX_BYTES: u64 = 128 * 1024 * 1024;
/// 图标缓存磁盘目录，相对于应用数据目录。
pub const ICON_CACHE_RELATIVE_DIR: &str = "icon-cache/apps";
