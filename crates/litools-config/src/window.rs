//! 窗口相关常量。
//!
//! `DEFAULT_` 前缀的常量未来可能开放给用户配置。

/// 主窗口默认宽度（像素），未来可由用户配置。
pub const DEFAULT_WINDOW_WIDTH: f64 = 820.0;
/// 主窗口默认高度（像素），未来可由用户配置。
pub const DEFAULT_WINDOW_HEIGHT: f64 = 560.0;
/// 分离窗口标题栏 webview 的高度（像素）。标题栏包含插件名称、关闭/停靠按钮。
pub const TITLEBAR_HEIGHT: f64 = 68.0;
/// 窗口 chrome 内边距：WindowFrame 的 1px 边框 + Panel 的 1px 边框。
/// 插件内容区域会内缩这个值，确保内容不贴边。
pub const CHROME_INSET: f64 = 2.0;
