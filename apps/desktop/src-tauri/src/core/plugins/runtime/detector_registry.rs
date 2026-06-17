use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use litools_core::context_analyzer::ContextAnalyzer;
use tauri::AppHandle;

use super::detection_bridge::WebviewDetectionBridge;
use super::js_detector_adapter::JsDetectorAdapter;

/// 已注册的检测器元数据
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RegisteredDetector {
    pub plugin_id: String,
    pub runtime_id: String,
    pub local_detector_id: String,
    pub feature_kind: String,
    pub detector_id: String,
    pub webview_label: String,
    pub registered_at: String,
}

/// 插件检测器注册表 —— 管理 JS Detector 的生命周期。
///
/// 与 `WebviewSearchBridge` 模式对称：
/// - 记录每个 detector 的归属（plugin_id + runtime_id）
/// - runtime 关闭时批量清理
/// - 插件卸载时批量清理
pub struct DetectorRegistry {
    registered: Mutex<HashMap<String, RegisteredDetector>>,
    context_analyzer: Arc<ContextAnalyzer>,
    detection_bridge: Arc<WebviewDetectionBridge>,
}

impl DetectorRegistry {
    pub fn new(
        context_analyzer: Arc<ContextAnalyzer>,
        detection_bridge: Arc<WebviewDetectionBridge>,
    ) -> Self {
        Self {
            registered: Mutex::new(HashMap::new()),
            context_analyzer,
            detection_bridge,
        }
    }

    /// 注册一个 WebView detector（同时写入 ContextAnalyzer + 元数据）。
    pub fn register_webview_detector(
        &self,
        info: RegisteredDetector,
        app_handle: AppHandle,
        timeout_ms: u64,
    ) {
        let detector_id = info.detector_id.clone();
        self.unregister(&detector_id);

        let detector = Arc::new(JsDetectorAdapter::new(
            info.detector_id.clone(),
            info.local_detector_id.clone(),
            info.feature_kind.clone(),
            info.runtime_id.clone(),
            info.webview_label.clone(),
            app_handle,
            self.detection_bridge.clone(),
            timeout_ms,
        ));
        self.context_analyzer.register_detector(detector);
        self.registered
            .lock()
            .unwrap()
            .insert(detector_id.clone(), info);
    }

    /// 注销单个 detector。
    pub fn unregister(&self, detector_id: &str) {
        self.context_analyzer.unregister_detector(detector_id);
        self.detection_bridge.cancel_detector(detector_id);
        self.registered.lock().unwrap().remove(detector_id);
    }

    /// 注销某个 runtime 下的所有 detector。
    pub fn unregister_runtime(&self, runtime_id: &str) -> Vec<String> {
        let ids: Vec<String> = {
            let reg = self.registered.lock().unwrap();
            reg.iter()
                .filter(|(_, info)| info.runtime_id == runtime_id)
                .map(|(id, _)| id.clone())
                .collect()
        };
        for id in &ids {
            self.context_analyzer.unregister_detector(id);
        }
        self.detection_bridge.cancel_runtime(runtime_id);
        {
            let mut reg = self.registered.lock().unwrap();
            for id in &ids {
                reg.remove(id);
            }
        }
        ids
    }

    /// 注销某个插件的所有 detector。
    #[allow(dead_code)]
    pub fn unregister_plugin(&self, plugin_id: &str) -> Vec<String> {
        let ids: Vec<String> = {
            let reg = self.registered.lock().unwrap();
            reg.iter()
                .filter(|(_, info)| info.plugin_id == plugin_id)
                .map(|(id, _)| id.clone())
                .collect()
        };
        for id in &ids {
            self.context_analyzer.unregister_detector(id);
        }
        {
            let mut reg = self.registered.lock().unwrap();
            for id in &ids {
                reg.remove(id);
            }
        }
        ids
    }
}
