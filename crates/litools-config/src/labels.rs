//! 窗口、webview 及实体 ID 标签前缀。
//!
//! 这些前缀用于运行时标识窗口/webview 实例，确保同一类型的标签不会冲突。
//! 命名规则：`{类型前缀}_{序号}`，如 `surface_000001`、`runtime_000001`。

// ── 窗口标签 ──

/// 主窗口标签。整个应用只有一个主窗口，标签固定为 `"main"`。
pub const MAIN_WINDOW_LABEL: &str = "main";
/// 分离面板窗口标签前缀，格式：`detached-panel_{序号}`。
pub const DETACHED_PANEL_WINDOW_PREFIX: &str = "detached-panel-";
/// 插件分离窗口标签前缀，格式：`plugin-window_{runtime_id}`。
pub const PLUGIN_WINDOW_PREFIX: &str = "plugin-window-";

// ── Webview 标签 ──

/// 插件内容 webview 标签前缀，格式：`plugin-{runtime_id}`。
pub const PLUGIN_WEBVIEW_PREFIX: &str = "plugin-";
/// 分离窗口标题栏 webview 标签前缀，格式：`titlebar-{runtime_id}`。
pub const TITLEBAR_WEBVIEW_PREFIX: &str = "titlebar-";
/// Surface webview 标签前缀，格式：`surface-{surface_id}`。
pub const SURFACE_WEBVIEW_LABEL_PREFIX: &str = "surface-";

// ── Registry ID 前缀 ──

/// Surface 注册表 ID 前缀，格式：`surface_{序号}`。
pub const SURFACE_ID_PREFIX: &str = "surface";
/// 插件运行时注册表 ID 前缀，格式：`runtime_{序号}`。
pub const RUNTIME_ID_PREFIX: &str = "runtime";
/// 分离宿主窗口注册表 ID 前缀，格式：`panel_{序号}`。
pub const DETACHED_HOST_ID_PREFIX: &str = "panel";

// ── View ID ──

/// 核心启动器视图 ID，前后端统一标识。
pub const CORE_LAUNCHER_VIEW_ID: &str = "core.launcher";
