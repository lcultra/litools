use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use litools_search::{RuntimeError, SearchProvider, SearchRuntime};
use tauri::AppHandle;

use super::search_bridge::{RegisteredSearchProvider, WebviewSearchBridge};
use super::search_provider::WebviewSearchProvider;

/// Hidden WebView 实现的搜索运行时。
///
/// AppHandle 在 Tauri setup 阶段通过 `set_app_handle()` 注入，
/// 因为 `AppState::bootstrap()` 在 Tauri 启动前执行，此时 AppHandle 尚不可用。
pub struct WebviewSearchRuntime {
    bridge: Arc<WebviewSearchBridge>,
    app_handle: Mutex<Option<AppHandle>>,
}

impl WebviewSearchRuntime {
    pub fn new(bridge: Arc<WebviewSearchBridge>) -> Self {
        Self {
            bridge,
            app_handle: Mutex::new(None),
        }
    }

    /// Tauri setup 阶段注入 AppHandle。
    pub fn set_app_handle(&self, app_handle: AppHandle) {
        *self.app_handle.lock().unwrap() = Some(app_handle);
    }
}

#[async_trait]
impl SearchRuntime for WebviewSearchRuntime {
    async fn register_provider(
        &self,
        plugin_id: &str,
        runtime_id: &str,
        local_provider_id: &str,
        provider_id: &str,
        webview_label: &str,
        timeout_ms: u64,
    ) -> Result<Arc<dyn SearchProvider>, RuntimeError> {
        let app_handle = self
            .app_handle
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| RuntimeError::Internal("AppHandle not set".to_string()))?;

        // 幂等 replace：先清理旧 provider
        self.bridge.unregister_provider(provider_id);

        let provider: Arc<dyn SearchProvider> = Arc::new(WebviewSearchProvider::new(
            provider_id.to_string(),
            local_provider_id.to_string(),
            runtime_id.to_string(),
            webview_label.to_string(),
            app_handle,
            self.bridge.clone(),
            timeout_ms,
        ));

        self.bridge.register_provider(
            RegisteredSearchProvider {
                plugin_id: plugin_id.to_string(),
                runtime_id: runtime_id.to_string(),
                provider_id: provider_id.to_string(),
                webview_label: webview_label.to_string(),
                registered_at: chrono::Utc::now().to_rfc3339(),
            },
            provider.clone(),
        );

        Ok(provider)
    }

    fn unregister_provider(&self, provider_id: &str) {
        self.bridge.unregister_provider(provider_id);
    }

    fn unregister_runtime(&self, runtime_id: &str) {
        self.bridge.unregister_runtime(runtime_id);
    }
}
