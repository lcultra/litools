use std::sync::Arc;

use litools_index::IndexDatabase;
use litools_plugin::{PluginCommand, PluginCommandMode, PluginManifest};
use crate::app_provider::{AppSearchProvider, app_id_from_result_id};
use crate::command::CommandExecution;
use crate::context::AppContext;
use crate::error::{LitoolsError, LitoolsResult};
use crate::executor_registry::ResultExecutor;
use crate::execution_service;
use crate::extension_registry::ExtensionRegistry;
use crate::internal_plugin::InternalPlugin;

/// 启动器内置插件：提供 App 搜索 + 启动能力。
pub struct LauncherPlugin {
    app_search_provider: Arc<AppSearchProvider>,
    app_executor: Arc<AppResultExecutor>,
    metadata: PluginManifest,
}

impl LauncherPlugin {
    pub fn new(database: IndexDatabase) -> Self {
        Self {
            app_search_provider: Arc::new(AppSearchProvider::new(database)),
            app_executor: Arc::new(AppResultExecutor),
            metadata: launcher_manifest(),
        }
    }
}

impl InternalPlugin for LauncherPlugin {
    fn metadata(&self) -> PluginManifest {
        self.metadata.clone()
    }

    fn register_extensions(&self, registry: &mut ExtensionRegistry) {
        registry.add_search_provider(self.app_search_provider.clone());
        registry.add_result_executor("apps", self.app_executor.clone());
    }
}

/// App 结果执行器：委托给 execution_service::execute_app。
pub struct AppResultExecutor;

impl ResultExecutor for AppResultExecutor {
    fn execute(
        &self,
        result_id: &str,
        action_id: &str,
        ctx: &AppContext,
    ) -> LitoolsResult<CommandExecution> {
        let app_id = app_id_from_result_id(result_id).ok_or_else(|| {
            LitoolsError::CommandNotFound(result_id.to_string())
        })?;
        execution_service::execute_app(
            &ctx.database,
            &ctx.system_adapter,
            result_id,
            app_id,
            action_id,
        )
    }
}

/// 插件清单（硬编码，plugin.json 仅作文档参考）。
fn launcher_manifest() -> PluginManifest {
    PluginManifest {
        id: "dev.litools.launcher".to_string(),
        name: "启动器".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        entry: None,
        description: Some("提供系统应用搜索与启动能力".to_string()),
        author: Some("litools".to_string()),
        icon: "launcher.svg".to_string(),
        commands: vec![PluginCommand {
            id: "search-apps".to_string(),
            title: "搜索应用".to_string(),
            subtitle: None,
            keywords: vec![],
            mode: PluginCommandMode::SearchProvider,
            executor: None,
            icon: None,
            script: None,
        }],
        singleton: true,
        permissions: vec![
            "litools-core:allow-apps-search".to_string(),
            "litools-core:allow-apps-launch".to_string(),
        ],
        development: None,
    }
}
