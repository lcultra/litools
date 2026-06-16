use std::sync::Arc;

use litools_plugin::{PluginManifest, PluginManager};

use crate::extension_registry::ExtensionRegistry;
use crate::internal_plugin::InternalPlugin;
use crate::plugin_provider::PluginCommandProvider;

/// 插件宿主编译插件：将已安装插件的 View 命令作为搜索结果暴露。
///
/// 包装 [`PluginCommandProvider`] 为 [`InternalPlugin`]，
/// 统一走 `register_internal_plugin()` 注册路径。
pub struct PluginHostPlugin {
    command_provider: Arc<PluginCommandProvider>,
    metadata: PluginManifest,
}

impl PluginHostPlugin {
    pub fn new(plugin_manager: Arc<PluginManager>) -> Self {
        let provider = Arc::new(PluginCommandProvider::new());
        provider.set_plugin_manager(plugin_manager);
        Self {
            command_provider: provider,
            metadata: plugin_host_manifest(),
        }
    }

    /// 获取内部 [`PluginCommandProvider`] 引用，供生命周期管理使用。
    pub fn command_provider(&self) -> Arc<PluginCommandProvider> {
        self.command_provider.clone()
    }
}

impl InternalPlugin for PluginHostPlugin {
    fn metadata(&self) -> PluginManifest {
        self.metadata.clone()
    }

    fn register_extensions(&self, registry: &mut ExtensionRegistry) {
        registry.add_search_provider(self.command_provider.clone());
    }
}

fn plugin_host_manifest() -> PluginManifest {
    PluginManifest {
        id: "dev.litools.plugin-host".to_string(),
        name: "插件宿主".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        entry: None,
        description: Some("将已安装插件的 View 命令暴露为搜索结果".to_string()),
        author: Some("litools".to_string()),
        icon: "plugin-host.svg".to_string(),
        commands: vec![],
        singleton: true,
        permissions: vec![],
        development: None,
    }
}
