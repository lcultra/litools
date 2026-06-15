pub mod webview;
pub mod background;

pub use webview::WebviewExecutor;
pub use background::BackgroundRuntimeExecutor;

use std::collections::HashMap;

/// 命令执行器 trait
pub trait CommandExecutor: Send + Sync {
    fn execute(&self, command: &ResolvedCommand, ctx: &ExecutionContext) -> Result<(), String>;
}

/// 解析后的命令（包含绝对路径）
#[derive(Debug, Clone)]
pub struct ResolvedCommand {
    pub id: String,
    pub plugin_id: String,
    pub command_id: String,
    pub title: String,
    pub mode: String,
    pub executor: String,
    pub icon: Option<String>,
    pub script: Option<String>,
    pub permissions: Vec<String>,
}

/// 执行上下文
pub struct ExecutionContext {
    pub runtime_id: Option<String>,
    pub webview_label: Option<String>,
    pub app_handle: tauri::AppHandle,
}

/// 执行器注册表（字符串 key → 实现）
pub struct ExecutorRegistry {
    executors: HashMap<String, Box<dyn CommandExecutor>>,
    mode_defaults: HashMap<String, String>,
}

impl ExecutorRegistry {
    pub fn new() -> Self {
        let mut mode_defaults = HashMap::new();
        mode_defaults.insert("view".to_string(), "webview".to_string());
        mode_defaults.insert("instant".to_string(), "backgroundRuntime".to_string());
        mode_defaults.insert("searchProvider".to_string(), "provider".to_string());

        Self {
            executors: HashMap::new(),
            mode_defaults,
        }
    }

    pub fn register(&mut self, name: &str, executor: Box<dyn CommandExecutor>) {
        self.executors.insert(name.to_string(), executor);
    }

    /// 解析命令的实际 executor：显式指定 > mode 默认映射 > "webview" fallback
    pub fn resolve_executor(&self, command: &ResolvedCommand) -> String {
        if !command.executor.is_empty() {
            return command.executor.clone();
        }
        self.mode_defaults
            .get(&command.mode)
            .cloned()
            .unwrap_or_else(|| "webview".to_string())
    }

    pub fn execute(
        &self,
        command: &ResolvedCommand,
        ctx: &ExecutionContext,
    ) -> Result<(), String> {
        let executor_name = self.resolve_executor(command);
        let executor = self
            .executors
            .get(&executor_name)
            .ok_or_else(|| format!("unknown executor: {executor_name}"))?;
        executor.execute(command, ctx)
    }
}
