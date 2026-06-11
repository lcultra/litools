use serde::Serialize;
use tauri::State;

use crate::{
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
    let app = state.app().lock().map_err(|error| error.to_string())?;
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
            icon: plugin.manifest.icon.clone(),
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

    let app = state.app().lock().map_err(|error| error.to_string())?;
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

    let mut app = state.app().lock().map_err(|e| e.to_string())?;
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
    let mut app = state.app().lock().map_err(|e| e.to_string())?;

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
    let mut app = state.app().lock().map_err(|e| e.to_string())?;
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
