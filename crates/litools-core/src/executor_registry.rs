use std::collections::HashMap;
use std::sync::Arc;

use crate::command::CommandExecution;
use crate::context::AppContext;
use crate::error::LitoolsResult;

/// 搜索结果执行器——按 provider_id 路由。
///
/// 每个 SearchProvider 可注册一个对应的 ResultExecutor，
/// ExecutorRegistry 通过 provider_id 找到正确的执行器。
pub trait ResultExecutor: Send + Sync {
    /// 执行某个搜索结果。
    fn execute(
        &self,
        result_id: &str,
        action_id: &str,
        ctx: &AppContext,
    ) -> LitoolsResult<CommandExecution>;
}

/// 搜索结果执行器注册表。
///
/// 按 provider_id 索引，provider_id 即 SearchResult.provider 字段值（如 "apps"）。
pub struct ExecutorRegistry {
    executors: HashMap<String, Arc<dyn ResultExecutor>>,
}

impl ExecutorRegistry {
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
        }
    }

    /// 注册一个执行器，provider_id 对应 SearchResult.provider。
    pub fn register(&mut self, provider_id: &str, executor: Arc<dyn ResultExecutor>) {
        self.executors
            .insert(provider_id.to_string(), executor);
    }

    /// 按 provider_id 查找执行器并执行。
    pub fn execute(
        &self,
        provider_id: &str,
        result_id: &str,
        action_id: &str,
        ctx: &AppContext,
    ) -> LitoolsResult<CommandExecution> {
        let executor = self
            .executors
            .get(provider_id)
            .ok_or_else(|| {
                crate::error::LitoolsError::CommandNotFound(result_id.to_string())
            })?;
        executor.execute(result_id, action_id, ctx)
    }
}
