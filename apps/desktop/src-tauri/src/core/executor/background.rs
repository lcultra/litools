use std::sync::Arc;

use crate::background::manager::BackgroundRuntimeManager;

use super::{CommandExecutor, ExecutionContext, ResolvedCommand};

/// Executor for mode=instant — runs script in background webview
pub struct BackgroundRuntimeExecutor {
    manager: Arc<BackgroundRuntimeManager>,
}

impl BackgroundRuntimeExecutor {
    pub fn new(manager: Arc<BackgroundRuntimeManager>) -> Self {
        Self { manager }
    }
}

impl CommandExecutor for BackgroundRuntimeExecutor {
    fn execute(&self, command: &ResolvedCommand, _ctx: &ExecutionContext) -> Result<(), String> {
        let script_uri = command
            .script
            .as_ref()
            .ok_or_else(|| format!("instant command '{}' has no script", command.id))?;
        self.manager.execute(&command.plugin_id, script_uri)
    }
}
