use serde::Serialize;
use tauri::{State, Webview};

use crate::{
    core::events::PluginEvent,
    core::plugins::runtime::service::find_enabled_plugin_command,
    protocol::plugin::resolve_entry_url, state::AppState,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCommandSummary {
    id: String,
    title: String,
    subtitle: Option<String>,
    keywords: Vec<String>,
    mode: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSummary {
    id: String,
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
    icon: String,
    enabled: bool,
    trusted: bool,
    source: String,
    path: String,
    permissions: Vec<String>,
    commands: Vec<PluginCommandSummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginViewDescriptor {
    plugin_id: String,
    command_id: String,
    plugin_name: String,
    title: String,
    entry_url: String,
    icon: String,
    permissions: Vec<String>,
    /// 是否处于开发模式（manifest 中有 development 字段）
    dev: bool,
}

#[tauri::command]
pub fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginSummary>, String> {
    let app = state.app().read().unwrap();
    Ok(app
        .context()
        .plugins
        .installed_plugins()
        .into_iter()
        .map(|plugin| PluginSummary {
            id: plugin.manifest.id.clone(),
            name: plugin.manifest.name.clone(),
            version: plugin.manifest.version.clone(),
            description: plugin.manifest.description.clone(),
            author: plugin.manifest.author.clone(),
            icon: format!("litools-plugin://{}/{}", plugin.manifest.id, plugin.manifest.icon),
            enabled: plugin.enabled,
            trusted: plugin.trusted,
            source: plugin.source.as_str().to_string(),
            path: plugin.path.display().to_string(),
            permissions: plugin.manifest.permissions.clone(),
            commands: plugin
                .manifest
                .commands
                .iter()
                .map(|command| PluginCommandSummary {
                    id: command.id.clone(),
                    title: command.title.clone(),
                    subtitle: command.subtitle.clone(),
                    keywords: command.keywords.clone(),
                    mode: command.mode.as_str().to_string(),
                })
                .collect(),
        })
        .collect())
}

#[tauri::command]
pub fn get_plugin_view_descriptor(
    plugin_id: String,
    command_id: String,
    state: State<'_, AppState>,
) -> Result<PluginViewDescriptor, String> {
    let (plugin_name, title, permissions, _policy) =
        find_enabled_plugin_command(&state, &plugin_id, &command_id)?;

    let app = state.app().read().unwrap();
    let plugin = app.context().plugins.find_plugin(&plugin_id).unwrap();
    let dev = plugin.manifest.development.is_some();
    let entry_url = resolve_entry_url(&plugin.manifest.id, &plugin.manifest)?;

    let icon = format!(
        "litools-plugin://{}/{}",
        plugin.manifest.id, plugin.manifest.icon
    );

    Ok(PluginViewDescriptor {
        plugin_id,
        command_id,
        plugin_name,
        title,
        entry_url,
        icon,
        permissions,
        dev,
    })
}

#[tauri::command]
pub fn install_plugin(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<serde_json::Value, String> {
    let data_dir = state.data_dir().to_path_buf();

    let src = std::path::PathBuf::from(&file_path);
    if !src.exists() {
        return Err(format!("文件不存在: {file_path}"));
    }

    // 解压到临时目录
    let tmp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let file = std::fs::File::open(&src).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    archive.extract(tmp_dir.path()).map_err(|e| e.to_string())?;

    // 查找 plugin.json
    let manifest_path = walkdir::WalkDir::new(tmp_dir.path())
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name() == "plugin.json")
        .map(|e| e.path().to_path_buf())
        .ok_or_else(|| "压缩包中未找到 plugin.json".to_string())?;

    let manifest_json = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let manifest: litools_plugin::PluginManifest =
        serde_json::from_str(&manifest_json).map_err(|e| e.to_string())?;
    manifest.validate().map_err(|e| e.to_string())?;

    let plugin_dir = manifest_path.parent().unwrap().to_path_buf();
    let dest_dir = data_dir.join("plugins").join(&manifest.id);

    // 复制到用户插件目录
    if dest_dir.exists() {
        std::fs::remove_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(&plugin_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = dest_dir.join(entry.file_name());
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }

    let mut app = state.app().write().unwrap();
    let now = chrono::Utc::now().to_rfc3339();
    let root_dir_str = dest_dir.to_string_lossy().to_string();

    // 写入 DB
    {
        let connection = app.context().database.connection();
        let plugins = litools_index::repository::PluginRepository::new(&connection);
        let manifest_json_str = serde_json::to_string(&manifest).map_err(|e| e.to_string())?;
        plugins
            .upsert_plugin(litools_index::repository::PluginUpsert {
                id: &manifest.id,
                name: &manifest.name,
                version: &manifest.version,
                path: &root_dir_str,
                manifest_json: &manifest_json_str,
                source: "user",
                enabled: true,
                trusted: false,
                installed_at: &now,
                updated_at: &now,
            })
            .map_err(|e| e.to_string())?;

        let command_upserts: Vec<_> = manifest
            .commands
            .iter()
            .map(|cmd| litools_index::repository::PluginCommandUpsert {
                id: litools_plugin::plugin_result_id(&manifest.id, &cmd.id),
                plugin_id: manifest.id.clone(),
                command_id: cmd.id.clone(),
                title: cmd.title.clone(),
                subtitle: cmd.subtitle.clone(),
                keywords: cmd.keywords.clone(),
                mode: cmd.mode.as_str().to_string(),
                executor: None,
                icon: None,
                script: None,
                source: "manifest".to_string(),
                lifecycle: "permanent".to_string(),
                registrar_runtime_id: None,
                executor_runtime_id: None,
                permission_requirements: manifest.permissions.clone(),
            })
            .collect();
        let commands_repo = litools_index::repository::PluginCommandRepository::new(&connection);
        commands_repo
            .replace_commands_for_plugin(&manifest.id, &command_upserts)
            .map_err(|e| e.to_string())?;
    }

    // 刷新插件管理器
    app.reload_plugins().map_err(|e| e.to_string())?;
    let summary = app
        .plugin_summary(&manifest.id)
        .ok_or_else(|| "安装后未找到插件".to_string())?;
    Ok(summary)
}

#[tauri::command]
pub fn uninstall_plugin(state: State<'_, AppState>, plugin_id: String) -> Result<(), String> {
    let mut app = state.app().write().unwrap();

    let plugin = app
        .context()
        .plugins
        .find_plugin(&plugin_id)
        .ok_or_else(|| format!("插件不存在: {plugin_id}"))?;
    let plugin_dir = plugin.path.clone();

    // 从 DB 删除
    {
        let connection = app.context().database.connection();
        connection
            .execute(
                "DELETE FROM plugin_commands WHERE plugin_id = ?1",
                rusqlite::params![&plugin_id],
            )
            .map_err(|e| e.to_string())?;
        connection
            .execute(
                "DELETE FROM plugins WHERE id = ?1",
                rusqlite::params![&plugin_id],
            )
            .map_err(|e| e.to_string())?;
    }

    // 删除文件
    let path = std::path::Path::new(&plugin_dir);
    if path.exists() {
        std::fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    }

    // 刷新插件管理器
    app.reload_plugins().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_plugin(
    state: State<'_, AppState>,
    plugin_id: String,
    enabled: bool,
) -> Result<serde_json::Value, String> {
    let mut app = state.app().write().unwrap();
    {
        let connection = app.context().database.connection();
        connection
            .execute(
                "UPDATE plugins SET enabled = ?1 WHERE id = ?2",
                rusqlite::params![enabled, &plugin_id],
            )
            .map_err(|e| e.to_string())?;
    }
    app.reload_plugins().map_err(|e| e.to_string())?;
    let summary = app
        .plugin_summary(&plugin_id)
        .ok_or_else(|| format!("插件不存在: {plugin_id}"))?;
    Ok(summary)
}

pub fn add_commands_inner(
    state: &AppState,
    webview: &Webview,
    commands: Vec<serde_json::Value>,
) -> Result<(), String> {
    let app = state.app().read().unwrap();
    let runtime_id = webview
        .label()
        .strip_prefix("plugin-")
        .ok_or("not a plugin webview")?
        .to_string();

    // 从 runtime registry 取 plugin_id
    let plugin_id = {
        let runtimes = state.plugin_runtimes.lock().map_err(|e| e.to_string())?;
        let rt = runtimes
            .runtime(&runtime_id)
            .ok_or_else(|| format!("runtime not found: {runtime_id}"))?;
        rt.plugin_id.clone()
    };

    let command_ids = {
        let connection = app.context().database.connection();
        let repo = litools_index::repository::PluginCommandRepository::new(&connection);
        let mut ids = Vec::new();

        for cmd in &commands {
            let id = cmd["id"].as_str().ok_or("missing id")?;
            let title = cmd["title"].as_str().ok_or("missing title")?;
            let command_id = format!("{}:{}", plugin_id, id);
            ids.push(command_id.clone());

            repo.upsert_command(&litools_index::repository::PluginCommandUpsert {
                id: command_id,
                plugin_id: plugin_id.clone(),
                command_id: id.to_string(),
                title: title.to_string(),
                subtitle: cmd["subtitle"].as_str().map(String::from),
                keywords: cmd["keywords"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                mode: cmd["mode"].as_str().unwrap_or("instant").to_string(),
                executor: cmd["executor"].as_str().map(String::from),
                icon: cmd["icon"].as_str().map(String::from),
                script: cmd["script"].as_str().map(String::from),
                source: "runtime".to_string(),
                lifecycle: cmd["lifecycle"].as_str().unwrap_or("permanent").to_string(),
                registrar_runtime_id: Some(runtime_id.clone()),
                executor_runtime_id: cmd["executor_runtime_id"].as_str().map(String::from),
                permission_requirements: vec![],
            })
            .map_err(|e| e.to_string())?;
        }
        ids
    }; // connection guard 在此释放

    drop(app);
    state
        .plugin_events
        .emit(PluginEvent::CommandsAdded(plugin_id, command_ids));
    Ok(())
}

#[tauri::command]
pub fn add_commands(
    state: State<'_, AppState>,
    webview: Webview,
    commands: Vec<serde_json::Value>,
) -> Result<(), String> {
    add_commands_inner(&state, &webview, commands)
}

pub fn remove_commands_inner(
    state: &AppState,
    webview: &Webview,
    ids: Vec<String>,
) -> Result<(), String> {
    let app = state.app().read().unwrap();
    let runtime_id = webview
        .label()
        .strip_prefix("plugin-")
        .ok_or("not a plugin webview")?
        .to_string();

    let plugin_id = {
        let runtimes = state.plugin_runtimes.lock().map_err(|e| e.to_string())?;
        let rt = runtimes
            .runtime(&runtime_id)
            .ok_or_else(|| format!("runtime not found: {runtime_id}"))?;
        rt.plugin_id.clone()
    };

    let command_ids: Vec<String> = ids
        .iter()
        .map(|id| litools_plugin::plugin_result_id(&plugin_id, id))
        .collect();

    {
        let connection = app.context().database.connection();
        for cid in &command_ids {
            connection
                .execute(
                    "DELETE FROM plugin_commands WHERE id = ?1 AND source = 'runtime' AND registrar_runtime_id = ?2",
                    rusqlite::params![cid, &runtime_id],
                )
                .map_err(|e| e.to_string())?;
        }
    } // connection guard 在此释放

    drop(app);
    state
        .plugin_events
        .emit(PluginEvent::CommandsRemoved(plugin_id, command_ids));
    Ok(())
}

#[tauri::command]
pub fn remove_commands(
    state: State<'_, AppState>,
    webview: Webview,
    ids: Vec<String>,
) -> Result<(), String> {
    remove_commands_inner(&state, &webview, ids)
}

pub fn replace_commands_inner(
    state: &AppState,
    webview: &Webview,
    commands: Vec<serde_json::Value>,
) -> Result<(), String> {
    let app = state.app().read().unwrap();
    let runtime_id = webview
        .label()
        .strip_prefix("plugin-")
        .ok_or("not a plugin webview")?
        .to_string();

    let plugin_id = {
        let runtimes = state.plugin_runtimes.lock().map_err(|e| e.to_string())?;
        let rt = runtimes
            .runtime(&runtime_id)
            .ok_or_else(|| format!("runtime not found: {runtime_id}"))?;
        rt.plugin_id.clone()
    };

    let count = {
        let connection = app.context().database.connection();

        // 原子：删 + 插
        connection
            .execute(
                "DELETE FROM plugin_commands WHERE source = 'runtime' AND registrar_runtime_id = ?1",
                rusqlite::params![&runtime_id],
            )
            .map_err(|e| e.to_string())?;

        let repo = litools_index::repository::PluginCommandRepository::new(&connection);
        let n = commands.len();
        for cmd in &commands {
            let id = cmd["id"].as_str().ok_or("missing id")?;
            let title = cmd["title"].as_str().ok_or("missing title")?;
            let command_id = litools_plugin::plugin_result_id(&plugin_id, id);

            repo.upsert_command(&litools_index::repository::PluginCommandUpsert {
                id: command_id,
                plugin_id: plugin_id.clone(),
                command_id: id.to_string(),
                title: title.to_string(),
                subtitle: cmd["subtitle"].as_str().map(String::from),
                keywords: cmd["keywords"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                mode: cmd["mode"].as_str().unwrap_or("instant").to_string(),
                executor: cmd["executor"].as_str().map(String::from),
                icon: cmd["icon"].as_str().map(String::from),
                script: cmd["script"].as_str().map(String::from),
                source: "runtime".to_string(),
                lifecycle: cmd["lifecycle"].as_str().unwrap_or("permanent").to_string(),
                registrar_runtime_id: Some(runtime_id.clone()),
                executor_runtime_id: None,
                permission_requirements: vec![],
            })
            .map_err(|e| e.to_string())?;
        }
        n
    }; // connection guard 在此释放

    drop(app);
    state
        .plugin_events
        .emit(PluginEvent::CommandsReplaced(plugin_id.clone(), count));
    Ok(())
}

#[tauri::command]
pub fn replace_commands(
    state: State<'_, AppState>,
    webview: Webview,
    commands: Vec<serde_json::Value>,
) -> Result<(), String> {
    replace_commands_inner(&state, &webview, commands)
}

pub fn update_command_inner(
    state: &AppState,
    webview: &Webview,
    id: &str,
    cmd: &serde_json::Value,
) -> Result<(), String> {
    let app = state.app().read().unwrap();
    let runtime_id = webview
        .label()
        .strip_prefix("plugin-")
        .ok_or("not a plugin webview")?
        .to_string();

    let plugin_id = {
        let runtimes = state.plugin_runtimes.lock().map_err(|e| e.to_string())?;
        let rt = runtimes
            .runtime(&runtime_id)
            .ok_or_else(|| format!("runtime not found: {runtime_id}"))?;
        rt.plugin_id.clone()
    };

    let command_id = litools_plugin::plugin_result_id(&plugin_id, id);

    {
        let connection = app.context().database.connection();
        let repo = litools_index::repository::PluginCommandRepository::new(&connection);

        // 读取现有命令
        let existing = repo
            .find_plugin_command(&plugin_id, id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("command not found: {command_id}"))?;

        // merge 部分更新
        let title = cmd
            .get("title")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or(existing.title);
        let subtitle = cmd
            .get("subtitle")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or(existing.subtitle);
        let keywords: Vec<String> = cmd
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or(existing.keywords);
        let mode = cmd
            .get("mode")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or(existing.mode);
        let executor = cmd
            .get("executor")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or(existing.executor);
        let icon = cmd
            .get("icon")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or(existing.icon);
        let script = cmd
            .get("script")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or(existing.script);
        let lifecycle = cmd
            .get("lifecycle")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or(existing.lifecycle);

        repo.upsert_command(&litools_index::repository::PluginCommandUpsert {
            id: command_id.clone(),
            plugin_id: plugin_id.clone(),
            command_id: id.to_string(),
            title,
            subtitle,
            keywords,
            mode,
            executor,
            icon,
            script,
            source: existing.source,
            lifecycle,
            registrar_runtime_id: existing.registrar_runtime_id,
            executor_runtime_id: existing.executor_runtime_id,
            permission_requirements: existing.permission_requirements,
        })
        .map_err(|e| e.to_string())?;
    } // connection guard 在此释放

    drop(app);
    state
        .plugin_events
        .emit(PluginEvent::CommandsUpdated(plugin_id, vec![command_id]));
    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = dst.join(entry.file_name());
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
