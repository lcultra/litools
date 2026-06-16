use litools_index::repository::PluginCommandRecord;

/// 解析后的命令（script 已转为绝对 URI）——当前为预留结构。
#[allow(dead_code)]
#[derive(Clone, Debug)]
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

#[allow(dead_code)]
fn default_executor(mode: &str) -> String {
    match mode {
        "view" => "webview",
        "instant" => "backgroundRuntime",
        "searchProvider" => "provider",
        _ => "webview",
    }
    .to_string()
}

/// 将 PluginCommandRecord 解析为 ResolvedCommand
/// script 字段存在则使用；否则 fallback 到 dist/commands/{command_id}.js
#[allow(dead_code)]
pub fn resolve_command(command: &PluginCommandRecord) -> ResolvedCommand {
    let script = command.script.clone().or_else(|| {
        Some(format!("dist/commands/{}.js", command.command_id))
    });

    let script_uri = script.map(|s| {
        format!(
            "litools-plugin://{}/{}",
            command.plugin_id,
            s.trim_start_matches('/')
        )
    });

    ResolvedCommand {
        id: command.id.clone(),
        plugin_id: command.plugin_id.clone(),
        command_id: command.command_id.clone(),
        title: command.title.clone(),
        mode: command.mode.clone(),
        executor: command
            .executor
            .clone()
            .filter(|e| !e.is_empty())
            .unwrap_or_else(|| default_executor(&command.mode)),
        icon: command.icon.clone(),
        script: script_uri,
        permissions: command.permission_requirements.clone(),
    }
}
