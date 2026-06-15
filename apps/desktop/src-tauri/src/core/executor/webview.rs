use super::{CommandExecutor, ExecutionContext, ResolvedCommand};

/// Executor for mode=view — delegates to existing dock_plugin_runtime
pub struct WebviewExecutor;

impl CommandExecutor for WebviewExecutor {
    fn execute(&self, _command: &ResolvedCommand, _ctx: &ExecutionContext) -> Result<(), String> {
        // View commands: the frontend handles navigation.
        // Backend just returns the OpenPluginView effect — this is already handled
        // in execute_result → CommandEffect::OpenPluginView.
        // No additional backend work needed here.
        Ok(())
    }
}
