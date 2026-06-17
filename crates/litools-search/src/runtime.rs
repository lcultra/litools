use std::sync::Arc;

use async_trait::async_trait;

use crate::SearchProvider;

/// 搜索运行时抽象 —— 管理 JS 侧 SearchProvider 的生命周期。
///
/// 当前唯一实现是 `WebviewSearchRuntime`（Hidden WebView），
/// 未来可替换为 QuickJS / WASM / Deno Isolate 等，
/// 插件代码和 SDK API 无需变更。
#[async_trait]
pub trait SearchRuntime: Send + Sync {
    /// 为此 webview 创建一个 SearchProvider 并注册到 SearchEngine。
    ///
    /// `provider_id` 是全局唯一的 provider 标识（通常为 `{plugin_id}.{custom_id}`）。
    /// `timeout_ms` 为单次搜索超时（毫秒）。
    ///
    /// 返回的 provider 已注册到 SearchEngine，可直接参与搜索。
    async fn register_provider(
        &self,
        plugin_id: &str,
        runtime_id: &str,
        local_provider_id: &str,
        provider_id: &str,
        webview_label: &str,
        timeout_ms: u64,
    ) -> std::result::Result<Arc<dyn SearchProvider>, RuntimeError>;

    /// 注销单个 provider（同时从 SearchEngine 移除）。
    fn unregister_provider(&self, provider_id: &str);

    /// 注销某个 runtime 下的所有 provider。
    fn unregister_runtime(&self, runtime_id: &str);
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("provider {0} already registered")]
    AlreadyRegistered(String),
    #[error("provider {0} not found")]
    NotFound(String),
    #[error("runtime error: {0}")]
    Internal(String),
}
