use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use litools_search::{SearchProvider, SearchRequest, SearchResult};
use serde_json::json;
use tauri::{AppHandle, Emitter};

use super::search_bridge::{SearchRequestId, WebviewSearchBridge};

/// 适配 WebView 插件的搜索提供者 —— 通过 IPC 桥回调 JS 侧注册的搜索函数
pub struct WebviewSearchProvider {
    provider_id: String,
    local_provider_id: String,
    runtime_id: String,
    webview_label: String,
    app_handle: AppHandle,
    bridge: Arc<WebviewSearchBridge>,
    timeout_ms: u64,
}

impl WebviewSearchProvider {
    pub fn new(
        provider_id: String,
        local_provider_id: String,
        runtime_id: String,
        webview_label: String,
        app_handle: AppHandle,
        bridge: Arc<WebviewSearchBridge>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            provider_id,
            local_provider_id,
            runtime_id,
            webview_label,
            app_handle,
            bridge,
            timeout_ms,
        }
    }
}

#[async_trait]
impl SearchProvider for WebviewSearchProvider {
    fn id(&self) -> &str {
        &self.provider_id
    }

    fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    async fn search(&self, request: &SearchRequest) -> Vec<SearchResult> {
        let trace_id = uuid::Uuid::new_v4();
        let request_id = SearchRequestId::new(&self.provider_id, trace_id);

        let rx = self
            .bridge
            .register_pending(request_id.clone(), self.runtime_id.clone());

        // 定向 emit 到目标 webview，携带完整 context
        let payload = json!({
            "requestId": request_id.to_string(),
            "providerId": self.provider_id,
            "localProviderId": self.local_provider_id,
            "query": request.query.text,
            "context": request.context,
        });
        let _ = self
            .app_handle
            .emit_to(&self.webview_label, "litools:search-request", payload);

        let timeout = Duration::from_millis(self.timeout_ms);
        tokio::select! {
            result = rx => {
                result.unwrap_or_default()
            }
            _ = tokio::time::sleep(timeout) => {
                self.bridge.cancel(&request_id);
                vec![]
            }
        }
    }
}
