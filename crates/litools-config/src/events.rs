//! 前端事件名称常量。
//!
//! 后端通过 Tauri 的 `emit` 向 webview 推送事件，前端通过 `listen` 订阅。
//! 所有事件名集中在此，确保前后端一致。

// ── Surface 事件 ──

/// 通知前端聚焦搜索输入框。切换回启动器视图时触发。
pub const FOCUS_SEARCH_EVENT: &str = "focus-search";
/// 通知前端导航到指定路由。打开插件/回到启动器时触发。
pub const NAVIGATE_EVENT: &str = "navigate";
/// 通知前端 Surface 元数据变更（生命周期、焦点、路由等）。
pub const SURFACE_METADATA_CHANGED_EVENT: &str = "surface-metadata-changed";

// ── 索引事件 ──

/// 索引状态变更通知（开始/完成/错误）。前端可据此刷新搜索结果。
pub const INDEX_STATUS_CHANGED_EVENT: &str = "index-status-changed";

// ── 插件运行时生命周期事件 ──

/// 插件 webview 获得焦点 / 切换到前台。注入的 `window.litools.lifecycle.onEnter` 会收到通知。
pub const LIFECYCLE_ENTER_EVENT: &str = "plugin-runtime://enter";
/// 插件 webview 失去焦点 / 切换到后台。注入的 `window.litools.lifecycle.onLeave` 会收到通知。
pub const LIFECYCLE_LEAVE_EVENT: &str = "plugin-runtime://leave";
