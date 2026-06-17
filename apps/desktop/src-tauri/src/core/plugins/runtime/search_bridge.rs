use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use litools_search::{SearchEngine, SearchResult};
use tokio::sync::oneshot;
use uuid::Uuid;

/// 搜索请求标识 —— 带 provider 信息，便于日志/调试
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct SearchRequestId {
    pub provider_id: String,
    pub nonce: Uuid,
}

impl SearchRequestId {
    pub fn new(provider_id: impl Into<String>, nonce: Uuid) -> Self {
        Self {
            provider_id: provider_id.into(),
            nonce,
        }
    }
}

impl std::fmt::Display for SearchRequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.provider_id, self.nonce)
    }
}

/// 待处理的搜索请求
pub struct PendingSearch {
    pub runtime_id: String,
    pub tx: oneshot::Sender<Vec<SearchResult>>,
}

/// 已注册的 Provider 信息
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RegisteredSearchProvider {
    pub plugin_id: String,
    pub runtime_id: String,
    pub provider_id: String,
    pub webview_label: String,
    pub registered_at: String,
}

/// WebView 搜索桥 —— 全局管理 pending requests 和 provider 注册表。
/// 作为 provider 注册的单一入口，同时维护 SearchEngine 和 lifecycle 元数据。
pub struct WebviewSearchBridge {
    pending: Mutex<HashMap<SearchRequestId, PendingSearch>>,
    registered: Mutex<HashMap<String, RegisteredSearchProvider>>,
    search_engine: Arc<SearchEngine>,
}

impl WebviewSearchBridge {
    pub fn new(search_engine: Arc<SearchEngine>) -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
            registered: Mutex::new(HashMap::new()),
            search_engine,
        }
    }

    /// 注册一个 pending 搜索请求
    pub fn register_pending(
        &self,
        request_id: SearchRequestId,
        runtime_id: String,
    ) -> oneshot::Receiver<Vec<SearchResult>> {
        let (tx, rx) = oneshot::channel();
        self.pending
            .lock()
            .unwrap()
            .insert(request_id, PendingSearch { runtime_id, tx });
        rx
    }

    /// 完成一个搜索请求（校验 runtime 归属）
    pub fn complete(
        &self,
        request_id: &SearchRequestId,
        runtime_id: &str,
        results: Vec<SearchResult>,
    ) -> bool {
        let mut pending = self.pending.lock().unwrap();
        if let Some(p) = pending.remove(request_id) {
            if p.runtime_id == runtime_id {
                let _ = p.tx.send(results);
                return true;
            }
        }
        false
    }

    /// 超时时清理 pending 请求
    pub fn cancel(&self, request_id: &SearchRequestId) {
        self.pending.lock().unwrap().remove(request_id);
    }

    /// 注册一个 provider（同时写入 SearchEngine）
    pub fn register_provider(
        &self,
        info: RegisteredSearchProvider,
        provider: Arc<dyn litools_search::SearchProvider>,
    ) -> Option<RegisteredSearchProvider> {
        self.search_engine
            .register_plugin_provider(&info.plugin_id, provider);
        self.registered
            .lock()
            .unwrap()
            .insert(info.provider_id.clone(), info)
    }

    /// 注销单个 provider（同时从 SearchEngine 移除）
    pub fn unregister_provider(&self, provider_id: &str) -> Option<RegisteredSearchProvider> {
        self.search_engine.unregister_provider(provider_id);
        self.registered.lock().unwrap().remove(provider_id)
    }

    /// 注销某个 runtime 下的所有 provider（同时从 SearchEngine 移除）
    pub fn unregister_runtime(&self, runtime_id: &str) -> Vec<String> {
        let ids = {
            let mut reg = self.registered.lock().unwrap();
            let ids: Vec<String> = reg
                .iter()
                .filter(|(_, info)| info.runtime_id == runtime_id)
                .map(|(id, _)| id.clone())
                .collect();
            for id in &ids {
                reg.remove(id);
            }
            ids
        };
        for id in &ids {
            self.search_engine.unregister_provider(id);
        }
        ids
    }

    /// 注销某个插件的所有 provider（同时从 SearchEngine 移除）
    #[allow(dead_code)]
    pub fn unregister_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.search_engine.unregister_plugin(plugin_id);
        let ids = {
            let mut reg = self.registered.lock().unwrap();
            let ids: Vec<String> = reg
                .iter()
                .filter(|(_, info)| info.plugin_id == plugin_id)
                .map(|(id, _)| id.clone())
                .collect();
            for id in &ids {
                reg.remove(id);
            }
            ids
        };
        ids
    }
}
