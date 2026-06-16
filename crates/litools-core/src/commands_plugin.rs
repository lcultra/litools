use std::sync::Arc;

use litools_plugin::{PluginCommand, PluginCommandMode, PluginManifest};

use crate::command::BuiltinCommandProvider;
use crate::extension_registry::ExtensionRegistry;
use crate::internal_plugin::InternalPlugin;

/// 内置命令插件：将硬编码的 5 个内置命令（重载索引、打开日志/数据目录、退出、切换主题）
/// 作为搜索 provider 暴露，统一走 InternalPlugin 注册路径。
pub struct CommandsPlugin {
    provider: Arc<BuiltinCommandProvider>,
    metadata: PluginManifest,
}

impl CommandsPlugin {
    pub fn new() -> Self {
        Self {
            provider: Arc::new(BuiltinCommandProvider),
            metadata: commands_manifest(),
        }
    }
}

impl InternalPlugin for CommandsPlugin {
    fn metadata(&self) -> PluginManifest {
        self.metadata.clone()
    }

    fn register_extensions(&self, registry: &mut ExtensionRegistry) {
        registry.add_search_provider(self.provider.clone());
    }
}

fn commands_manifest() -> PluginManifest {
    PluginManifest {
        id: "dev.litools.commands".to_string(),
        name: "内置命令".to_string(),
        version: "1.0.0".to_string(),
        entry: None,
        description: Some("提供重载索引、打开目录、退出、切换主题等内置命令".to_string()),
        author: Some("litools contributors".to_string()),
        icon: "commands.svg".to_string(),
        commands: builtin_commands(),
        singleton: true,
        permissions: vec![
            "litools-core:allow-index".to_string(),
            "litools-core:allow-file-manager".to_string(),
            "litools-core:allow-settings".to_string(),
        ],
        development: None,
    }
}

fn builtin_commands() -> Vec<PluginCommand> {
    vec![
        PluginCommand {
            id: "reload-index".to_string(),
            title: "重载索引".to_string(),
            subtitle: Some("刷新本地搜索索引".to_string()),
            keywords: vec!["reload".to_string(), "index".to_string(), "refresh".to_string(), "rebuild".to_string()],
            mode: PluginCommandMode::Instant,
            executor: None, icon: None, script: None,
        },
        PluginCommand {
            id: "open-logs-directory".to_string(),
            title: "打开日志目录".to_string(),
            subtitle: Some("在系统文件管理器中打开日志目录".to_string()),
            keywords: vec!["logs".to_string(), "log".to_string(), "directory".to_string(), "folder".to_string(), "debug".to_string()],
            mode: PluginCommandMode::Instant,
            executor: None, icon: None, script: None,
        },
        PluginCommand {
            id: "open-data-directory".to_string(),
            title: "打开数据目录".to_string(),
            subtitle: Some("在系统文件管理器中打开本地数据目录".to_string()),
            keywords: vec!["data".to_string(), "directory".to_string(), "folder".to_string(), "storage".to_string(), "database".to_string()],
            mode: PluginCommandMode::Instant,
            executor: None, icon: None, script: None,
        },
        PluginCommand {
            id: "quit-app".to_string(),
            title: "退出应用".to_string(),
            subtitle: Some("退出 litools".to_string()),
            keywords: vec!["quit".to_string(), "exit".to_string(), "close".to_string()],
            mode: PluginCommandMode::Instant,
            executor: None, icon: None, script: None,
        },
        PluginCommand {
            id: "toggle-theme".to_string(),
            title: "切换主题".to_string(),
            subtitle: Some("在浅色和深色主题之间切换".to_string()),
            keywords: vec!["theme".to_string(), "toggle".to_string(), "dark".to_string(), "light".to_string()],
            mode: PluginCommandMode::Instant,
            executor: None, icon: None, script: None,
        },
    ]
}
