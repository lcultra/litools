//! 后台运行时管理器——为 Instant 模式插件提供后台 WebView 执行环境。
//! 当前为预留架构，待 Instant 命令功能完整实现后激活。

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::runtime::PluginRuntime;

struct RuntimeEntry {
    runtime: Box<dyn PluginRuntime>,
    last_used: Instant,
    #[allow(dead_code)]
    plugin_id: String,
}

pub struct BackgroundRuntimeManager {
    runtimes: Mutex<HashMap<String, RuntimeEntry>>,
    policy: RuntimePolicy,
    idle_timeout: Duration,
}

#[derive(Clone, Copy)]
pub enum RuntimePolicy {
    #[allow(dead_code)]
    Shared,
    PerPlugin,
}

impl BackgroundRuntimeManager {
    pub fn new(policy: RuntimePolicy, idle_timeout: Duration) -> Self {
        Self {
            runtimes: Mutex::new(HashMap::new()),
            policy,
            idle_timeout,
        }
    }

    #[allow(dead_code)]
    pub fn register_runtime(
        &self,
        plugin_id: &str,
        runtime: Box<dyn PluginRuntime>,
    ) {
        let key = self.resolve_key(plugin_id);
        let mut runtimes = self.runtimes.lock().unwrap();
        runtimes.insert(
            key,
            RuntimeEntry {
                runtime,
                last_used: Instant::now(),
                plugin_id: plugin_id.to_string(),
            },
        );
    }

    pub fn execute(
        &self,
        plugin_id: &str,
        script_uri: &str,
    ) -> Result<(), String> {
        let key = self.resolve_key(plugin_id);
        let mut runtimes = self.runtimes.lock().unwrap();

        // 清理空闲超时的 runtime
        let timeout = self.idle_timeout;
        runtimes.retain(|_, entry| entry.last_used.elapsed() < timeout);

        if let Some(entry) = runtimes.get_mut(&key) {
            entry.last_used = Instant::now();
            entry.runtime.execute(script_uri)
        } else {
            Err(format!(
                "no runtime registered for plugin '{}' (key: '{}')",
                plugin_id, key
            ))
        }
    }

    fn resolve_key(&self, plugin_id: &str) -> String {
        match self.policy {
            RuntimePolicy::Shared => "shared".to_string(),
            RuntimePolicy::PerPlugin => plugin_id.to_string(),
        }
    }
}
