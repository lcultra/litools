use std::sync::Arc;

use litools_plugin::PluginManifest;
use litools_search::SearchProvider;

use crate::executor_registry::ResultExecutor;

/// 内置插件的运行时表示。
///
/// 插件实例通过 trait 方法暴露能力（search_providers / result_executors），
/// 不依赖 plugin.json 驱动构造。plugin.json 仅作为元数据声明存在。
pub trait InternalPlugin: Send + Sync {
    /// 插件元数据（从 plugin.json 加载或硬编码）。
    fn metadata(&self) -> PluginManifest;

    /// 插件提供的搜索能力（provider_id 由 provider.id() 决定）。
    fn search_providers(&self) -> Vec<Arc<dyn SearchProvider>> {
        vec![]
    }

    /// 插件提供的结果执行器，返回 (provider_id, executor) 对。
    fn result_executors(&self) -> Vec<(String, Arc<dyn ResultExecutor>)> {
        vec![]
    }
}
