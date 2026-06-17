use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use litools_search::{Detection, InputDetector};
use serde_json::json;
use tauri::{AppHandle, Emitter};

use super::detection_bridge::WebviewDetectionBridge;

/// JS Detector 适配器 —— 将 JS 侧的检测函数包装为 `InputDetector`。
///
/// 通过 IPC 向插件 WebView 发送检测请求，等待 JS 侧返回结果。
/// 与 `WebviewSearchProvider` 模式对称：emit 事件 → oneshot 等待 → 返回结果。
pub struct JsDetectorAdapter {
    detector_id: String,
    local_detector_id: String,
    feature_kind: String,
    source: String,
    runtime_id: String,
    webview_label: String,
    app_handle: AppHandle,
    bridge: Arc<WebviewDetectionBridge>,
    timeout: Duration,
}

impl JsDetectorAdapter {
    pub fn new(
        detector_id: String,
        local_detector_id: String,
        feature_kind: String,
        runtime_id: String,
        webview_label: String,
        app_handle: AppHandle,
        bridge: Arc<WebviewDetectionBridge>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            source: format!("plugin.{}", detector_id),
            detector_id,
            local_detector_id,
            feature_kind,
            runtime_id,
            webview_label,
            app_handle,
            bridge,
            timeout: Duration::from_millis(timeout_ms),
        }
    }
}

#[async_trait]
impl InputDetector for JsDetectorAdapter {
    fn id(&self) -> &str {
        &self.detector_id
    }

    fn feature_kind(&self) -> &str {
        &self.feature_kind
    }

    fn source(&self) -> Option<&str> {
        Some(&self.source)
    }

    async fn detect(&self, input: &str) -> Option<Detection> {
        let (request_id, rx) = self
            .bridge
            .register_pending(&self.detector_id, self.runtime_id.clone());

        let payload = json!({
            "requestId": format!("{}.{}", request_id.detector_id, request_id.nonce),
            "detectorId": self.detector_id,
            "localDetectorId": self.local_detector_id,
            "input": input,
        });
        let _ = self
            .app_handle
            .emit_to(&self.webview_label, "litools:detection-request", payload);

        tokio::select! {
            result = rx => {
                result.unwrap_or(None)
            }
            _ = tokio::time::sleep(self.timeout) => {
                self.bridge.cancel(&request_id);
                None
            }
        }
    }
}
