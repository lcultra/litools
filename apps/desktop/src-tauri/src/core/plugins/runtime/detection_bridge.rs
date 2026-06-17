use std::collections::HashMap;
use std::sync::Mutex;

use litools_search::Detection;
use tokio::sync::oneshot;
use uuid::Uuid;

/// 检测请求标识
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct DetectionRequestId {
    pub detector_id: String,
    pub nonce: Uuid,
}

impl DetectionRequestId {
    pub fn new(detector_id: impl Into<String>) -> Self {
        Self {
            detector_id: detector_id.into(),
            nonce: Uuid::new_v4(),
        }
    }
}

/// 待处理的检测请求
struct PendingDetection {
    runtime_id: String,
    tx: oneshot::Sender<Option<Detection>>,
}

/// WebView 检测桥 —— 管理 JS Detector 的 IPC 请求/响应。
pub struct WebviewDetectionBridge {
    pending: Mutex<HashMap<DetectionRequestId, PendingDetection>>,
}

impl WebviewDetectionBridge {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }

    /// 注册一个 pending 检测请求，返回 receiver。
    pub fn register_pending(
        &self,
        detector_id: &str,
        runtime_id: String,
    ) -> (DetectionRequestId, oneshot::Receiver<Option<Detection>>) {
        let request_id = DetectionRequestId::new(detector_id);
        let (tx, rx) = oneshot::channel();
        self.pending
            .lock()
            .unwrap()
            .insert(request_id.clone(), PendingDetection { runtime_id, tx });
        (request_id, rx)
    }

    /// 完成一个检测请求。
    pub fn complete(
        &self,
        request_id: &DetectionRequestId,
        runtime_id: &str,
        detection: Option<Detection>,
    ) -> bool {
        let mut pending = self.pending.lock().unwrap();
        if let Some(p) = pending.remove(request_id) {
            if p.runtime_id == runtime_id {
                let _ = p.tx.send(detection);
                return true;
            }
        }
        false
    }

    /// 超时时取消 pending 请求。
    pub fn cancel(&self, request_id: &DetectionRequestId) {
        self.pending.lock().unwrap().remove(request_id);
    }

    /// 注销单个 detector 时取消其所有 pending 请求。
    pub fn cancel_detector(&self, detector_id: &str) {
        self.pending
            .lock()
            .unwrap()
            .retain(|request_id, _| request_id.detector_id != detector_id);
    }

    /// 注销某个 runtime 下的所有 pending 请求。
    pub fn cancel_runtime(&self, runtime_id: &str) {
        self.pending
            .lock()
            .unwrap()
            .retain(|_, p| p.runtime_id != runtime_id);
    }
}
