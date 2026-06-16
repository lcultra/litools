use std::sync::Arc;

use litools_search::SearchProvider;

use crate::executor_registry::ResultExecutor;

/// 插件扩展注册表——插件通过此注册表声明其提供的各种能力。
///
/// 新增扩展类型只需给 `ExtensionRegistry` 加 `add_*` 方法，
/// 不会影响已有插件实现。遵循开闭原则。
pub struct ExtensionRegistry {
    search_providers: Vec<Arc<dyn SearchProvider>>,
    result_executors: Vec<(String, Arc<dyn ResultExecutor>)>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            search_providers: Vec::new(),
            result_executors: Vec::new(),
        }
    }

    /// 注册一个搜索提供者，provider_id 由 provider.id() 决定。
    pub fn add_search_provider(&mut self, provider: Arc<dyn SearchProvider>) {
        self.search_providers.push(provider);
    }

    /// 注册一个结果执行器，按 provider_id 路由。
    pub fn add_result_executor(
        &mut self,
        provider_id: impl Into<String>,
        executor: Arc<dyn ResultExecutor>,
    ) {
        self.result_executors.push((provider_id.into(), executor));
    }

    // ── 消费型访问器，供 bootstrap 使用 ──

    /// 拆解注册表，返回注册的 search_providers 和 result_executors。
    pub fn decompose(
        self,
    ) -> (
        Vec<Arc<dyn SearchProvider>>,
        Vec<(String, Arc<dyn ResultExecutor>)>,
    ) {
        (self.search_providers, self.result_executors)
    }
}
