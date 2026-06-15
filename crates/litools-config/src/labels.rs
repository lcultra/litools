//! 窗口、webview 及视图定义标签/标识。
//!
//! 标签命名规则：
//! - 窗口标签：`main-window`（固定）或 `detach-window-{uuid}`
//! - Webview 标签（即 surface_id）：`core-webview-{uuid}` / `plugin-webview-{uuid}`
//! - 视图定义 ID：`core.launcher`（固定，前后端共用）

// ── 窗口标签 ──

/// 主窗口标签。整个应用只有一个主窗口，标签固定为 `"main-window"`。
pub const MAIN_WINDOW_LABEL: &str = "main-window";
/// 分离窗口标签前缀，格式：`detach-window-{uuid}`。
pub const DETACH_WINDOW_PREFIX: &str = "detach-window-";

// ── Webview 标签（即 surface_id） ──

/// Core provider 的 webview / surface 标签前缀，格式：`core-webview-{uuid}`。
pub const CORE_WEBVIEW_PREFIX: &str = "core-webview-";
/// Plugin provider 的 webview / surface 标签前缀，格式：`plugin-webview-{uuid}`。
pub const PLUGIN_WEBVIEW_PREFIX: &str = "plugin-webview-";

// ── View ID ──

/// 核心启动器视图定义 ID，前后端统一标识。
pub const CORE_LAUNCHER_VIEW_ID: &str = "core.launcher";
